use serde::Serialize;
use std::io::prelude::*;
use std::io::Result;
use std::sync::Mutex;
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Debug, Serialize, Clone, PartialEq)]
#[repr(u16)]
pub enum MessageHeader {
    Confirm = 0x0302,
    SerialOut = 0x0303,
    VersionOut = 0x0307,
    Tick = 0x0400,
    SetBrightness = 0x0409,
    ConfirmFrameBuffer = 0x0410,
    SetVibration = 0x041b,
    ButtonPress = 0x0500,
    KnobRotate = 0x0501,
    Reset = 0x0506,
    SetColor = 0x0702,
    TouchDown = 0x094d,
    TouchUp = 0x096d,
    VersionIn = 0x0c07,
    MCU = 0x180d,
    SerialIn = 0x1f03,
    WriteFrameBuffer = 0xff10,
}

impl MessageHeader {
    pub fn from_u16(value: u16) -> Option<MessageHeader> {
        match value {
            0x0302 => Some(MessageHeader::Confirm),
            0x0303 => Some(MessageHeader::SerialOut),
            0x0307 => Some(MessageHeader::VersionOut),
            0x0400 => Some(MessageHeader::Tick),
            0x0409 => Some(MessageHeader::SetBrightness),
            0x0410 => Some(MessageHeader::ConfirmFrameBuffer),
            0x041b => Some(MessageHeader::SetVibration),
            0x0500 => Some(MessageHeader::ButtonPress),
            0x0501 => Some(MessageHeader::KnobRotate),
            0x0506 => Some(MessageHeader::Reset),
            0x0702 => Some(MessageHeader::SetColor),
            0x094d => Some(MessageHeader::TouchDown),
            0x096d => Some(MessageHeader::TouchUp),
            0x0c07 => Some(MessageHeader::VersionIn),
            0x180d => Some(MessageHeader::MCU),
            0x1f03 => Some(MessageHeader::SerialIn),
            0xff10 => Some(MessageHeader::WriteFrameBuffer),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[repr(u8)]
pub enum Button {
    Knob0 = 0x01,
    Knob1 = 0x02,
    Knob2 = 0x03,
    Knob3 = 0x04,
    Knob4 = 0x05,
    Knob5 = 0x06,
    Home = 0x07,
    Circle1 = 0x08,
    Circle2 = 0x09,
    Circle3 = 0x0a,
    Circle4 = 0x0b,
    Circle5 = 0x0c,
    Circle6 = 0x0d,
    Circle7 = 0x0e,
}

impl Button {
    pub fn from_u8(value: u8) -> Option<Button> {
        match value {
            0x01 => Some(Button::Knob0),
            0x02 => Some(Button::Knob1),
            0x03 => Some(Button::Knob2),
            0x04 => Some(Button::Knob3),
            0x05 => Some(Button::Knob4),
            0x06 => Some(Button::Knob5),
            0x07 => Some(Button::Home),
            0x08 => Some(Button::Circle1),
            0x09 => Some(Button::Circle2),
            0x0a => Some(Button::Circle3),
            0x0b => Some(Button::Circle4),
            0x0c => Some(Button::Circle5),
            0x0d => Some(Button::Circle6),
            0x0e => Some(Button::Circle7),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[repr(u8)]
pub enum Screen {
    Left,
    Center,
    Right,
}

impl Screen {
    pub fn from_x_coor(x: u16) -> Option<Screen> {
        match x {
            0..=60 => Some(Screen::Left),
            61..=420 => Some(Screen::Center),
            421..=480 => Some(Screen::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[repr(u8)]
pub enum Knob {
    Knob0 = 0x01,
    Knob1 = 0x02,
    Knob2 = 0x03,
    Knob3 = 0x04,
    Knob4 = 0x05,
    Knob5 = 0x06,
}

impl Knob {
    pub fn from_u8(value: u8) -> Option<Knob> {
        match value {
            0x01 => Some(Knob::Knob0),
            0x02 => Some(Knob::Knob1),
            0x03 => Some(Knob::Knob2),
            0x04 => Some(Knob::Knob3),
            0x05 => Some(Knob::Knob4),
            0x06 => Some(Knob::Knob5),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[repr(u8)]
pub enum PressDirection {
    Down = 0x00,
    Up = 0x01,
}

impl PressDirection {
    pub fn from_u8(value: u8) -> Option<PressDirection> {
        match value {
            0x00 => Some(PressDirection::Up),
            0x01 => Some(PressDirection::Down),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ButtonPressEvent {
    pub tx_id: u8,
    pub button: Button,
    pub dir: PressDirection,
}

#[derive(Debug, Serialize, Clone)]
pub struct KnobRotateEvent {
    pub tx_id: u8,
    pub knob: Knob,
    pub value: i8,
}

#[derive(Debug, Serialize, Clone)]
pub struct TouchEvent {
    pub tx_id: u8,
    pub dir: PressDirection,
    pub touch_id: u8,
    pub x: u16,
    pub y: u16,
    pub screen: Screen,
}

#[derive(Debug, Serialize, Clone)]
pub enum Event {
    ButtonPress(ButtonPressEvent),
    KnobRotate(KnobRotateEvent),
    TouchEvent(TouchEvent),
}

fn parse_serial_message(message: &[u8]) -> Result<Event> {
    let header: u16 = u16::from_be_bytes([message[0], message[1]]);
    let message_type = MessageHeader::from_u16(header).expect("Invalid header type");
    let tx_id = message[2];

    match message_type {
        MessageHeader::ButtonPress => {
            let button = Button::from_u8(message[3]).expect("Invalid button address");
            let dir = PressDirection::from_u8(message[4]).expect("Invalid button direction");
            Ok(Event::ButtonPress(ButtonPressEvent { button, dir, tx_id }))
        }
        MessageHeader::KnobRotate => {
            let knob = Knob::from_u8(message[3]).expect("Invalid knob address");
            Ok(Event::KnobRotate(KnobRotateEvent {
                tx_id,
                knob,
                value: message[4] as i8,
            }))
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

            Ok(Event::TouchEvent(TouchEvent {
                tx_id,
                dir,
                x,
                y,
                touch_id,
                screen,
            }))
        }
        _ => {
            println!("unknown header: {:02x}", header);
            print!("message: ");
            for m in message.iter() {
                print!("{:02x}", m);
            }
            println!("");
            Ok(Event::ButtonPress(ButtonPressEvent {
                tx_id,
                button: Button::Home,
                dir: PressDirection::Up,
            }))
        }
    }
}

const WS_UPGRADE_HEADER: &str = "GET /index.html
HTTP/1.1
Connection: Upgrade
Upgrade: websocket
Sec-WebSocket-Key: 123abc

";

// const WS_UPGRADE_RESPONSE: &str = "HTTP/1.1";

pub struct Device {
    pub port: String,
    runtime: Option<Runtime>,
}

impl Device {
    pub fn new(port: String) -> Device {
        Device {
            port,
            runtime: Some(Runtime::new().unwrap()),
        }
    }

    pub fn connect(&self, on_event: Option<Box<dyn Fn(Event) + Send + Sync + 'static>>) {
        println!("Connecting to Loupedeck on port {}", self.port);

        let mut port = serialport::new(&self.port, 9600)
            .open()
            .expect("Failed to open port");

        println!("Connected to Loupedeck on port {}", self.port);

        port.write(WS_UPGRADE_HEADER.as_bytes())
            .expect("Failed to write to port");
        self.poll(Mutex::new(port), Mutex::new(on_event));
    }

    pub fn disconnect(&mut self) {
        println!("Disconnecting from Loupedeck on port {}", self.port);

        let runtime = self.runtime.take();

        runtime
            .expect("Foo")
            .shutdown_timeout(Duration::from_millis(100));
    }

    fn poll(
        &self,
        port_mutex: Mutex<Box<dyn serialport::SerialPort>>,
        on_event_mutex: Mutex<Option<Box<dyn Fn(Event) + Send + Sync + 'static>>>,
    ) {
        match &self.runtime {
            Some(runtime) => {
                runtime.spawn(async move {
                    loop {
                        let mut port = port_mutex.lock().expect("Failed to lock port");

                        let mut single_byte: Vec<u8> = vec![0; 1];
                        port.read(single_byte.as_mut_slice())
                            .expect("Found no data!");

                        if single_byte[0] == 0x82 {
                            // Found delimiter
                            port.read(single_byte.as_mut_slice())
                                .expect("Found no length data");
                            let length = single_byte[0] as usize;
                            // println!("Found message with length {}", length);
                            let mut data: Vec<u8> = vec![0; length];
                            port.read(data.as_mut_slice()).expect("Found no data!");
                            let event = parse_serial_message(&data);

                            let on_event = on_event_mutex.lock().expect("Failed to lock on_event");

                            if event.is_ok() {
                                let event = event.unwrap();
                                if let Some(on_event) = on_event.as_ref() {
                                    on_event(event);
                                }
                            }
                        }
                    }
                });
            }
            None => {
                println!("No runtime");
            }
        }
    }
}

const LOUPEDECK_VENDOR_ID: u16 = 11970;

pub fn get_loupedeck_ports() -> Vec<String> {
    let mut ports = Vec::new();

    serialport::available_ports()
        .unwrap()
        .iter()
        .for_each(|port| {
            if let serialport::SerialPortType::UsbPort(port_info) = &port.port_type {
                if port_info.vid == LOUPEDECK_VENDOR_ID {
                    ports.push(port.port_name.to_string());
                }
            }
        });

    return ports;
}

pub fn connect_loupedeck_device<F>(port: String, on_event: F) -> Device
where
    F: Fn(Event) + Send + Sync + 'static,
{
    let loupedeck = Device::new(port);
    loupedeck.connect(Some(Box::new(on_event)));
    return loupedeck;
}
