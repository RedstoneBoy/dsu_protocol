#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Magic {
    Server,
    Client,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Protocol {
    Version1001,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageType {
    ProtocolVersionInfo,
    ControllerInfo,
    ControllerData,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    Disconnected,
    Reserved,
    Connected,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Model {
    NotApplicable,
    PartialGyro,
    FullGyro,
    Unused,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConnectionType {
    NotApplicable,
    Usb,
    Bluetooth,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BatteryStatus {
    NotApplicable,
    Dying,
    Low,
    Medium,
    High,
    Full,
    Charging,
    Charged,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Registration {
    AllControllers,
    SlotBased,
    MacBased,
}

#[derive(Copy, Clone, Debug)]
pub struct Buttons(pub(crate) [u8; 2]);

impl Buttons {
    pub fn new() -> Self {
        Buttons([0; 2])
    }

    pub fn clear(&mut self) {
        self.0 = [0; 2];
    }
}

impl std::ops::BitOr<Button> for Buttons {
    type Output = Buttons;

    fn bitor(mut self, rhs: Button) -> Buttons {
        let (bit, index) = rhs.bit_and_index();
        self.0[index] |= 1 << bit;
        self
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Button {
    Left,
    Down,
    Right,
    Up,
    Start,
    RStick,
    LStick,
    Select,
    Y,
    B,
    A,
    X,
    R1,
    L1,
    R2,
    L2,
}

impl Button {
    fn bit_and_index(&self) -> (u8, usize) {
        match self {
            Button::Left => (7, 0),
            Button::Down => (6, 0),
            Button::Right => (5, 0),
            Button::Up => (4, 0),
            Button::Start => (3, 0),
            Button::RStick => (2, 0),
            Button::LStick => (1, 0),
            Button::Select => (0, 0),
            Button::Y => (7, 1),
            Button::B => (6, 1),
            Button::A => (5, 1),
            Button::X => (4, 1),
            Button::R1 => (3, 1),
            Button::L1 => (2, 1),
            Button::R2 => (1, 1),
            Button::L2 => (0, 1),
        }
    }
}