use raqote::DrawTarget;
use std::io::prelude::*;
use std::io::Result;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError;
use tokio::time;
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::SerialStream;

mod plugin;
pub use plugin::*;

mod constants;
pub use constants::*;

fn parse_serial_message(message: &[u8]) -> Result<Option<Event>> {
    let header: u16 = u16::from_be_bytes([message[0], message[1]]);
    // println!("Message type: {:?}", header);

    let message_type = MessageHeader::from(header);
    let tx_id = message[2];

    match message_type {
        MessageHeader::ButtonPress => {
            let button = Button::from_u8(message[3]).expect("Invalid button address");
            let dir = PressDirection::from_u8(message[4]).expect("Invalid button direction");
            Ok(Some(Event::ButtonPress(ButtonPressEvent {
                button,
                dir,
                tx_id,
            })))
        }
        MessageHeader::KnobRotate => {
            let knob = Knob::from_u8(message[3]).expect("Invalid knob address");
            Ok(Some(Event::KnobRotate(KnobRotateEvent {
                tx_id,
                knob,
                value: message[4] as i8,
            })))
        }
        MessageHeader::TouchDown | MessageHeader::TouchUp => {
            let x = u16::from_be_bytes([message[4], message[5]]);
            let y = u16::from_be_bytes([message[6], message[7]]);
            let touch_id = message[8];
            let dir = if message_type == MessageHeader::TouchDown {
                PressDirection::Down
            } else {
                PressDirection::Up
            };
            let screen = Screen::from_x_coor(x).expect("Invalid screen");

            Ok(Some(Event::TouchEvent(TouchEvent {
                tx_id,
                dir,
                x,
                y,
                touch_id,
                screen,
            })))
        }
        MessageHeader::SerialIn => Ok(Some(Event::SerialIn(SerialInEvent {
            tx_id,
            serial_number: String::from_utf8(message[3..].to_vec())
                .unwrap()
                .trim()
                .to_string(),
        }))),
        MessageHeader::VersionIn => Ok(Some(Event::VersionIn(VersionInEvent {
            tx_id,
            version: format!("{}.{}.{}", message[3], message[4], message[5]).to_string(),
        }))),
        MessageHeader::ConfirmFrameBuffer => Ok(Some(Event::ConfirmFrameBufferIn(
            ConfirmFrameBufferInEvent { tx_id },
        ))),
        MessageHeader::DrawIn => Ok(Some(Event::DrawIn(DrawInEvent { tx_id }))),
        _ => Ok(Some(Event::Other(FallbackEvent { tx_id }))),
    }
}

fn get_tx_id(evt: Event) -> Option<u8> {
    match evt {
        Event::ButtonPress(ButtonPressEvent { tx_id, .. }) => Some(tx_id),
        Event::KnobRotate(KnobRotateEvent { tx_id, .. }) => Some(tx_id),
        Event::TouchEvent(TouchEvent { tx_id, .. }) => Some(tx_id),
        Event::Other(FallbackEvent { tx_id }) => Some(tx_id),
        Event::SerialIn(SerialInEvent { tx_id, .. }) => Some(tx_id),
        Event::VersionIn(VersionInEvent { tx_id, .. }) => Some(tx_id),
        Event::ConfirmFrameBufferIn(ConfirmFrameBufferInEvent { tx_id }) => Some(tx_id),
        Event::DrawIn(DrawInEvent { tx_id }) => Some(tx_id),
        _ => panic!("Invalid event"),
    }
}

const WS_UPGRADE_HEADER: &str = "GET /index.html
HTTP/1.1
Connection: Upgrade
Upgrade: websocket
Sec-WebSocket-Key: 123abc

";

const WS_UPGRADE_RESPONSE: &str = "HTTP/1.1";

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    serial: String,
    version: String,
}

pub struct Device {
    pub port: String,
    pub(crate) runtime: Option<Runtime>,
    tx_pending_send: Option<mpsc::Sender<Vec<u8>>>,
    pub tx_event: Option<broadcast::Sender<Event>>,
    next_tx_id: u8,
}

fn construct_draw_buffer_payload(
    screen: Screen,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    buffer: &[u8],
) -> Result<Vec<u8>> {
    if (width * height * 2) as usize != buffer.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid buffer length",
        ));
    }

    let mut buff = Vec::new();
    buff.append(&mut Vec::from(screen));
    buff.append(&mut x.to_be_bytes().to_vec());
    buff.append(&mut y.to_be_bytes().to_vec());
    buff.append(&mut width.to_be_bytes().to_vec());
    buff.append(&mut height.to_be_bytes().to_vec());
    buff.append(&mut buffer.to_vec());

    return Ok(buff);
}

fn construct_message_payload(message_buffer: Vec<u8>, tx_id: u8) -> Vec<u8> {
    let mut prefix_buff: Vec<u8>;

    if message_buffer.len() > 0xff {
        prefix_buff = Vec::with_capacity(14);

        for i in 0..14 {
            prefix_buff.push(0x00);
        }

        prefix_buff[0] = 0x82;
        prefix_buff[1] = 0xFF;

        let offset = 6;
        // println!("Message length: {:?}", message_buffer.len());

        (message_buffer.len() as u32)
            .to_be_bytes()
            .iter()
            .enumerate()
            .for_each(|(i, b)| {
                prefix_buff[offset + i] = *b;
            });

        // prefix_buff[2 + offset] = (len >> 24) as u8;
        // prefix_buff[3 + offset] = (len >> 16) as u8;
        // prefix_buff[4 + offset] = (len >> 8) as u8;
        // prefix_buff[5 + offset] = len as u8;
    } else {
        prefix_buff = Vec::with_capacity(6);

        for i in 0..6 {
            prefix_buff.push(0x00);
        }

        prefix_buff[0] = 0x82;
        prefix_buff[1] = 0x80 + message_buffer.len() as u8;
    }

    let mut message = Vec::with_capacity(prefix_buff.len() + message_buffer.len());
    message.extend_from_slice(&prefix_buff);
    message.extend_from_slice(&message_buffer);

    message[prefix_buff.len() + 2] = tx_id;

    message
}

fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

#[derive(Debug, Clone)]
pub struct ExternalDeviceEventEmitter {
    tx_event: mpsc::Sender<ExternalMessage>,
}

#[derive(Debug, Clone)]
pub struct ExternalMessage {
    action: Vec<u8>,
    data: Vec<u8>,
}

impl From<ExternalMessage> for Vec<u8> {
    fn from(external_message: ExternalMessage) -> Vec<u8> {
        let mut header: Vec<u8> = Vec::with_capacity(3);
        header.push(external_message.action[0]);
        header.push(external_message.action[1]);
        header.push(0x01);

        let mut message: Vec<u8> = Vec::with_capacity(header.len() + external_message.data.len());
        message.extend_from_slice(&header);
        message.extend_from_slice(&external_message.data);

        return construct_message_payload(message, 1);
    }
}

impl ExternalDeviceEventEmitter {
    fn new(tx_event: mpsc::Sender<ExternalMessage>) -> Self {
        Self { tx_event }
    }

    async fn send_message(&self, message: ExternalMessage) -> Result<()> {
        println!("Sending message: {:?}", message);
        self.tx_event.send(message).await;
        Ok(())
    }

    pub async fn vibrate(&self, level: Haptic) -> Result<()> {
        let mut vib_header = Vec::new();
        vib_header.push(0x04);
        vib_header.push(0x1b);

        let mut data = Vec::new();
        data.push(level as u8);

        self.send_message(ExternalMessage {
            action: vib_header,
            data,
        })
        .await;

        Ok(())
    }
}

impl Device {
    pub fn new(port: String) -> Device {
        Device {
            port,
            runtime: None,
            tx_pending_send: None,
            tx_event: None,
            next_tx_id: 2,
        }
    }

    pub fn create_external_event_emitter(&self) -> ExternalDeviceEventEmitter {
        let runtime = self.runtime.as_ref().unwrap();
        let (tx_ext_message, mut rx_ext_message): (
            mpsc::Sender<ExternalMessage>,
            mpsc::Receiver<ExternalMessage>,
        ) = mpsc::channel(10);

        let tx_pending_send = self.tx_pending_send.as_ref().unwrap().clone();

        runtime.spawn(async move {
            loop {
                // Try to write everything we can from the port
                while let Ok(ext_message) = rx_ext_message.try_recv() {
                    let message = Vec::from(ext_message);
                    println!("Sending external message: {:?}", to_hex(&message));
                    tx_pending_send.send(message).await.unwrap();
                }
            }
        });

        ExternalDeviceEventEmitter::new(tx_ext_message)
    }

    pub async fn connect(&mut self) -> Result<()> {
        if self.runtime.is_some() {
            return Ok(());
        }

        println!("Connecting to Loupedeck on port {}", self.port);

        let mut port = tokio_serial::new(self.port.clone(), 9600)
            .open_native_async()
            .expect("Failed to open port");

        println!("Waiting for writable");
        port.writable().await;

        port.try_write(WS_UPGRADE_HEADER.as_bytes())
            .expect("Failed to write to port");

        println!("Waiting for readable");
        port.readable().await;
        let mut buf = vec![0; 1024];

        port.try_read(buf.as_mut_slice())
            .expect("Failed to read from port");

        let res = String::from_utf8(buf).unwrap();

        if !res.contains(WS_UPGRADE_RESPONSE) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to upgrade to websocket",
            ));
        }

        self.start_polling(port);

        return Ok(());
    }

    pub fn disconnect(&mut self) {
        println!("Disconnecting from loupedeck on port {}", self.port);
        self.tx_event = None;
        self.tx_pending_send = None;

        let runtime = self.runtime.take();

        if let Some(runtime) = runtime {
            runtime.shutdown_background();
        }
    }

    pub async fn vibrate(&mut self, level: Haptic) {
        let mut vib_header = Vec::new();
        vib_header.push(0x04);
        vib_header.push(0x1b);

        let mut data = Vec::new();
        data.push(level as u8);

        self.send_message(vib_header, data, false).await;
    }

    pub async fn draw_key(&mut self, key_x: u16, key_y: u16, img: Vec<u8>) {
        let x: u16 = KEY_SIZE * key_x;
        let y: u16 = KEY_SIZE * key_y;
        let width: u16 = 90;
        let height: u16 = 90;

        self.draw_buffer(Screen::Center, x, y, width, height, img.as_slice())
            .await;
    }

    pub async fn draw_buffer(
        &mut self,
        screen: Screen,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        buffer: &[u8],
    ) -> Result<()> {
        let buff_res = construct_draw_buffer_payload(screen.clone(), x, y, width, height, buffer);

        if buff_res.is_err() {
            return Err(buff_res.unwrap_err());
        }

        self.send_message(
            Vec::from(MessageHeader::WriteFrameBuffer),
            buff_res.unwrap(),
            true,
        )
        .await;

        self.refresh_screen(screen).await;

        Ok(())
    }

    async fn refresh_screen(&mut self, screen: Screen) {
        self.send_message(Vec::from(MessageHeader::DrawOut), Vec::from(screen), true)
            .await;
    }

    pub async fn get_info(&mut self) -> Result<DeviceInfo> {
        let serial_evt = self
            .send_message(Vec::from(MessageHeader::SerialOut), vec![], true)
            .await;
        let version_evt = self
            .send_message(Vec::from(MessageHeader::VersionOut), vec![], true)
            .await;

        let mut version_number: Option<String> = None;
        let mut serial: Option<String> = None;

        if serial_evt.is_some() {
            let serial_evt_result = serial_evt.unwrap();
            if serial_evt_result.is_ok() {
                let serial_evt_event = serial_evt_result.unwrap();
                if let Event::SerialIn(SerialInEvent { serial_number, .. }) = serial_evt_event {
                    serial = Some(serial_number);
                }
            }
        }

        if version_evt.is_some() {
            let version_in_result = version_evt.unwrap();
            if version_in_result.is_ok() {
                let version_in_event = version_in_result.unwrap();
                if let Event::VersionIn(VersionInEvent { version, .. }) = version_in_event {
                    version_number = Some(version);
                }
            }
        }

        if version_number.is_none() || serial.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to get device info",
            ));
        }

        return Ok(DeviceInfo {
            serial: serial.unwrap(),
            version: version_number.unwrap(),
        });
    }

    async fn send_message(
        &mut self,
        action: Vec<u8>,
        data: Vec<u8>,
        expect_event: bool,
    ) -> Option<std::result::Result<Event, RecvError>> {
        let mut header: Vec<u8> = Vec::with_capacity(3);
        header.push(action[0]);
        header.push(action[1]);
        header.push(0x01);

        let mut message: Vec<u8> = Vec::with_capacity(header.len() + data.len());
        message.extend_from_slice(&header);
        message.extend_from_slice(&data);

        return self.send(message, expect_event).await;
    }

    async fn send(
        &mut self,
        message_buffer: Vec<u8>,
        expect_event: bool,
    ) -> Option<std::result::Result<Event, RecvError>> {
        let (response_tx, response_rx): (oneshot::Sender<Event>, oneshot::Receiver<Event>) =
            oneshot::channel();

        let mut tx_id: u8 = 0x01;
        if expect_event {
            tx_id = self.next_tx_id;
            self.next_tx_id += 1;
        }

        let runtime = self.runtime.as_ref().unwrap();
        let tx_pending_send = self.tx_pending_send.as_ref().unwrap().clone();
        let message = construct_message_payload(message_buffer, tx_id);

        if self.tx_event.is_some() && expect_event {
            let mut rx_event = self.tx_event.clone().unwrap().subscribe();

            runtime.spawn(async move {
                while let Ok(res) = rx_event.recv().await {
                    let evt_tx_id = get_tx_id(res.clone());

                    if evt_tx_id.is_some() && evt_tx_id.unwrap() == tx_id {
                        response_tx.send(res).unwrap();
                        break;
                    }
                }
            });
        }

        runtime.spawn(async move {
            tx_pending_send.send(message).await;
        });

        if expect_event {
            return Some(response_rx.await);
        } else {
            return None;
        }
    }

    fn start_polling(&mut self, mut serial: SerialStream) {
        let (tx_event, mut rx_event) = broadcast::channel(10);
        let (tx_pending_send, mut rx_pending_send) = mpsc::channel(100);
        self.runtime = Some(Runtime::new().unwrap());

        self.tx_pending_send = Some(tx_pending_send.clone());
        self.tx_event = Some(tx_event.clone());

        let runtime = self.runtime.as_ref().unwrap();

        runtime.spawn(async move {
            loop {
                // Send any pending messages

                // Try to write everything we can from the port
                while let Ok(message) = rx_pending_send.try_recv() {
                    println!(
                        "[LOOP] sending pending message {:?}",
                        to_hex(message.as_slice())
                    );
                    serial
                        .write(message.as_slice())
                        .expect("Failed to write to port");
                }

                loop {
                    let mut single_byte: Vec<u8> = vec![0; 1];
                    let res = serial.try_read(single_byte.as_mut_slice());

                    if res.is_err() {
                        break;
                    }

                    if single_byte[0] == 0x82 {
                        // Found delimiter
                        serial
                            .try_read(single_byte.as_mut_slice())
                            .expect("Found no length data");
                        let length = single_byte[0] as usize;
                        // println!("Found message with length {}", length);
                        let mut data: Vec<u8> = vec![0; length];
                        serial
                            .try_read(data.as_mut_slice())
                            .expect("Found no data!");
                        let event = parse_serial_message(&data);

                        if event.is_ok() {
                            let evt_unrw = event.unwrap();
                            if evt_unrw.is_some() {
                                tx_event.send(evt_unrw.unwrap()).unwrap();
                            }
                        }
                    }
                }

                time::sleep(time::Duration::from_millis(1)).await
            }
        });
    }
}

const LOUPEDECK_VENDOR_ID: u16 = 11970;

pub fn get_loupedeck_ports() -> Vec<String> {
    let mut ports = Vec::new();

    tokio_serial::available_ports()
        .unwrap()
        .iter()
        .for_each(|port| {
            if let tokio_serial::SerialPortType::UsbPort(port_info) = &port.port_type {
                if port_info.vid == LOUPEDECK_VENDOR_ID {
                    ports.push(port.port_name.to_string());
                }
            }
        });

    return ports;
}

#[cfg(test)]
mod tests {
    use super::construct_draw_buffer_payload;

    #[test]
    fn it_create_draw_buffer_red_key_payload() {
        let mut half_red: Vec<u8> = Vec::with_capacity(90 * 90 * 2);
        for i in 0..(90 * 90) {
            half_red.push(0x00);
            half_red.push(0xF8);
        }

        let mut payload = construct_draw_buffer_payload(
            crate::Screen::Center,
            180,
            0,
            90,
            90,
            half_red.as_slice(),
        )
        .expect("Failed to create payload");

        let header = vec![0x00, 0x41, 0x00, 0xb4, 0x00, 0x00, 0x00, 0x5a, 0x00, 0x5a];

        assert_eq!(header, payload.drain(0..header.len()).collect::<Vec<u8>>());
        // assert_eq!(payload.len(), 16213);
    }
}

pub fn convert_draw_target_to_rgb565(dt: DrawTarget) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let bg_color: u32 = 0x000000;

    // orig is AARRGGBB

    let draw_data = dt.get_data();
    for px in draw_data {
        let rgb16 = convert_argb888_to_rgb565(*px);
        result.append(&mut rgb16.to_le_bytes().to_vec());
    }

    result
}

fn convert_argb888_to_rgb565(argb: u32) -> u16 {
    let r = ((argb >> 16) & 0xff) as u16;
    let g = ((argb >> 8) & 0xff) as u16;
    let b = (argb & 0xff) as u16;
    ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)
}

#[cfg(test)]
mod draw_tests {
    use raqote::{DrawOptions, DrawTarget, SolidSource, Source};

    use crate::convert_draw_target_to_rgb565;

    use super::convert_argb888_to_rgb565;

    #[test]
    fn it_converts() {
        assert_eq!(convert_argb888_to_rgb565(0xFF000000), 0);
        assert_eq!(convert_argb888_to_rgb565(0xFFFFFFFF), 0xffff);
        assert_eq!(convert_argb888_to_rgb565(0xFF0000FF), 0x001f);
        assert_eq!(convert_argb888_to_rgb565(0xFFFF0000), 0xf800);
        assert_eq!(convert_argb888_to_rgb565(0xFF00FF00), 0x07e0);
    }

    #[test]
    fn it_works_for_dt_red() {
        let mut dt = DrawTarget::new(1, 1);

        let mut solidRed: SolidSource =
            SolidSource::from_unpremultiplied_argb(255, 0xFF, 0x00, 0x00);

        dt.fill_rect(
            0.0,
            0.0,
            1.0,
            1.0,
            &Source::Solid(solidRed),
            &DrawOptions::new(),
        );

        assert_eq!(convert_draw_target_to_rgb565(dt), vec![0x00, 0xF8,]);
    }
    #[test]
    fn it_works_for_dt_green() {
        let mut dt = DrawTarget::new(1, 1);

        let mut solidGreen: SolidSource =
            SolidSource::from_unpremultiplied_argb(255, 0x00, 0xFF, 0x00);

        dt.fill_rect(
            0.0,
            0.0,
            1.0,
            1.0,
            &Source::Solid(solidGreen),
            &DrawOptions::new(),
        );

        assert_eq!(convert_draw_target_to_rgb565(dt), vec![0xE0, 0x07,]);
    }

    #[test]
    fn it_works_for_dt_blue() {
        let mut dt = DrawTarget::new(1, 1);

        let mut solidBlue: SolidSource =
            SolidSource::from_unpremultiplied_argb(255, 0x00, 0x00, 0xFF);

        dt.fill_rect(
            0.0,
            0.0,
            1.0,
            1.0,
            &Source::Solid(solidBlue),
            &DrawOptions::new(),
        );

        assert_eq!(convert_draw_target_to_rgb565(dt), vec![0x1F, 0x00]);
    }
}
