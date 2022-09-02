use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
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
    DrawOut = 0x050F,
    DrawIn = 0x040F,
    SetColor = 0x0702,
    TouchDown = 0x094d,
    TouchUp = 0x096d,
    VersionIn = 0x0c07,
    MCU = 0x180d,
    SerialIn = 0x1f03,
    WriteFrameBuffer = 0xff10,
}

impl From<u16> for MessageHeader {
    fn from(value: u16) -> MessageHeader {
        match value {
            0x0302 => MessageHeader::Confirm,
            0x0303 => MessageHeader::SerialOut,
            0x0307 => MessageHeader::VersionOut,
            0x0400 => MessageHeader::Tick,
            0x0409 => MessageHeader::SetBrightness,
            0x0410 => MessageHeader::ConfirmFrameBuffer,
            0x041b => MessageHeader::SetVibration,
            0x0500 => MessageHeader::ButtonPress,
            0x0501 => MessageHeader::KnobRotate,
            0x050F => MessageHeader::DrawOut,
            0x040F => MessageHeader::DrawIn,
            0x0506 => MessageHeader::Reset,
            0x0702 => MessageHeader::SetColor,
            0x094d => MessageHeader::TouchDown,
            0x096d => MessageHeader::TouchUp,
            0x0c07 => MessageHeader::VersionIn,
            0x180d => MessageHeader::MCU,
            0x1f03 => MessageHeader::SerialIn,
            0xff10 => MessageHeader::WriteFrameBuffer,
            _ => panic!("Unknown message header: {}", value),
        }
    }
}

impl From<MessageHeader> for Vec<u8> {
    fn from(value: MessageHeader) -> Vec<u8> {
        vec![((value.clone() as u16) >> 8) as u8, value.clone() as u8]
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
#[repr(u16)]
pub enum Screen {
    Left,
    Center,
    Right,
}

impl From<Screen> for u16 {
    fn from(value: Screen) -> u16 {
        match value {
            Screen::Left => 0x004C,
            Screen::Center => 0x0041,
            Screen::Right => 0x0052,
        }
    }
}

impl From<Screen> for Vec<u8> {
    fn from(value: Screen) -> Vec<u8> {
        let base = u16::from(value);
        vec![((base.clone() as u16) >> 8) as u8, base.clone() as u8]
    }
}

struct Display {
    id: u16,
    width: u16,
    height: u16,
}

// See https://github.com/foxxyz/loupedeck/blob/master/constants.js#L50
#[derive(Debug, Serialize, Clone)]
#[repr(u8)]
pub enum Haptic {
    Short = 0x01,
    Medium = 0x0a,
    Long = 0x0f,
    Low = 0x31,
    ShortLow = 0x32,
    ShortLower = 0x33,
    Lower = 0x40,
    Lowest = 0x41,
    DescendSlow = 0x46,
    DescendMed = 0x47,
    DescendFast = 0x48,
    AscendSlow = 0x52,
    AscendMed = 0x53,
    AscendFast = 0x58,
    RevSlowest = 0x5e,
    RevSlow = 0x5f,
    RevMed = 0x60,
    RevFast = 0x61,
    RevFaster = 0x62,
    RevFastest = 0x63,
    RiseFall = 0x6a,
    Buzz = 0x70,
    Rumble5 = 0x77, // lower frequencies in descending order
    Rumble4 = 0x78,
    Rumble3 = 0x79,
    Rumble2 = 0x7a,
    Rumble1 = 0x7b,
    VeryLong = 0x76, // 10 sec high freq (!)
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
    Up = 0x00,
    Down = 0x01,
}

impl PressDirection {
    pub fn from_u8(value: u8) -> Option<PressDirection> {
        match value {
            0x00 => Some(PressDirection::Down),
            0x01 => Some(PressDirection::Up),
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
pub struct FallbackEvent {
    pub tx_id: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct SerialInEvent {
    pub tx_id: u8,
    pub serial_number: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ConfirmFrameBufferInEvent {
    pub tx_id: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct VersionInEvent {
    pub tx_id: u8,
    pub version: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct DrawInEvent {
    pub tx_id: u8,
}

#[derive(Debug, Serialize, Clone)]
pub enum Event {
    ButtonPress(ButtonPressEvent),
    KnobRotate(KnobRotateEvent),
    TouchEvent(TouchEvent),
    Other(FallbackEvent),
    SerialIn(SerialInEvent),
    VersionIn(VersionInEvent),
    ConfirmFrameBufferIn(ConfirmFrameBufferInEvent),
    DrawIn(DrawInEvent),
}

pub static KEY_SIZE: u16 = 90;
