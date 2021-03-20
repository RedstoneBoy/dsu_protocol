use core::convert::TryInto;

pub mod error;

use error::*;

pub const MESSAGE_PROTOCOL: u32 = 0x100000;
pub const MESSAGE_INFO: u32 = 0x100001;
pub const MESSAGE_DATA: u32 = 0x100002;

pub const PROTOCOL_VERSION_1001: u16 = 1001;

pub const MAGIC_CLIENT: [u8; 4] = [b'D', b'S', b'U', b'C'];
pub const MAGIC_SERVER: [u8; 4] = [b'D', b'S', b'U', b'S'];

pub const BUFFER_SIZE: usize = 100;

const CRC_POSITION: usize = 8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Magic {
    /// Used when the server is sending the message
    Server,
    /// Used when the client is sending the message
    Client,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProtocolVersion {
    Version1001,
}

impl ProtocolVersion {
    const LENGTH: usize = 2;

    fn serialize(&self) -> [u8; Self::LENGTH] {
        match self {
            ProtocolVersion::Version1001 => PROTOCOL_VERSION_1001.to_le_bytes(),
        }
    }

    fn deserialize(data: &[u8; Self::LENGTH]) -> Result<Self, u16> {
        let version = u16::from_le_bytes(*data);
        match version {
            PROTOCOL_VERSION_1001 => Ok(Self::Version1001),
            _ => Err(version),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageType {
    ProtocolVersionInfo,
    ControllerInfo,
    ControllerData,
}

#[derive(Clone, Debug)]
pub struct MessageHeader {
    pub magic: Magic,
    pub protocol_version: ProtocolVersion,
    /// Length of packet without header
    /// Note: message_type is not actually part of the header
    pub length: u16,
    /// CRC32 Hash of entire packet while crc was zeroed
    pub crc32_hash: u32,
    /// ID of client or server who sent this packet
    pub sender_id: u32,
    /// Note: this is not actually part of the header
    pub message_type: MessageType,
}

impl MessageHeader {
    pub const LENGTH: usize = 20;

    pub fn serialize(&self) -> [u8; Self::LENGTH] {
        let mut data = [0; Self::LENGTH];
        self.serialize_to(&mut data);
        data
    }

    pub fn serialize_to(&self, data: &mut [u8; Self::LENGTH]) {
        match self.magic {
            Magic::Client => data[0..4].copy_from_slice(&MAGIC_CLIENT),
            Magic::Server => data[0..4].copy_from_slice(&MAGIC_SERVER),
        }

        data[4..6].copy_from_slice(&self.protocol_version.serialize());

        data[6..8].copy_from_slice(&self.length.to_le_bytes());
        data[8..12].copy_from_slice(&self.crc32_hash.to_le_bytes());
        data[12..16].copy_from_slice(&self.sender_id.to_le_bytes());

        match self.message_type {
            MessageType::ProtocolVersionInfo => {
                data[16..20].copy_from_slice(&MESSAGE_PROTOCOL.to_le_bytes())
            }
            MessageType::ControllerInfo => {
                data[16..20].copy_from_slice(&MESSAGE_INFO.to_le_bytes())
            }
            MessageType::ControllerData => {
                data[16..20].copy_from_slice(&MESSAGE_DATA.to_le_bytes())
            }
        }
    }

    pub fn deserialize(data: &[u8; Self::LENGTH]) -> Result<Self, HeaderError> {
        let magic = match &data[0..4] {
            magic if magic == &MAGIC_CLIENT => Magic::Client,
            magic if magic == &MAGIC_SERVER => Magic::Server,
            magic => return Err(HeaderError::InvalidMagic(magic.try_into().unwrap())),
        };

        let protocol_version = ProtocolVersion::deserialize(&data[4..6].try_into().unwrap())
            .map_err(HeaderError::UnsupportedProtocolVersion)?;

        let length = u16::from_le_bytes(data[6..8].try_into().unwrap());
        let crc32_hash = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let sender_id = u32::from_le_bytes(data[12..16].try_into().unwrap());

        let message_type = match u32::from_le_bytes(data[16..20].try_into().unwrap()) {
            MESSAGE_PROTOCOL => MessageType::ProtocolVersionInfo,
            MESSAGE_INFO => MessageType::ControllerInfo,
            MESSAGE_DATA => MessageType::ControllerData,
            message_type => return Err(HeaderError::InvalidMessageType(message_type)),
        };

        Ok(MessageHeader {
            magic,
            protocol_version,
            length,
            crc32_hash,
            sender_id,
            message_type,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RequestProtocolInfoMessage;

impl RequestProtocolInfoMessage {
    const LENGTH: usize = 0;
}

#[derive(Clone, Debug)]
pub struct ProtocolInfoMessage {
    /// Maximum protocol version supported by your application
    pub protocol: ProtocolVersion,
}

impl ProtocolInfoMessage {
    pub const LENGTH: usize = 2;

    pub fn serialize(&self) -> [u8; Self::LENGTH] {
        let mut data = [0; Self::LENGTH];
        self.serialize_to(&mut data);
        data
    }

    pub fn serialize_to(&self, data: &mut [u8; Self::LENGTH]) {
        data.copy_from_slice(&self.protocol.serialize());
    }

    pub fn deserialize(data: &[u8; Self::LENGTH]) -> Result<Self, UnsupportedProtocolVersion> {
        ProtocolVersion::deserialize(data)
            .map(|protocol| ProtocolInfoMessage { protocol })
            .map_err(UnsupportedProtocolVersion)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SlotState {
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

#[derive(Clone, Debug)]
pub struct ControllerInfo {
    pub slot: u8,
    pub slot_state: SlotState,
    pub model: Model,
    pub connection_type: ConnectionType,
    pub mac: [u8; 6],
    pub battery_status: BatteryStatus,
}

impl ControllerInfo {
    const LENGTH: usize = 11;

    fn serialize_to(&self, data: &mut [u8; Self::LENGTH]) {
        data[0] = self.slot;
        data[1] = match self.slot_state {
            SlotState::Disconnected => 0,
            SlotState::Reserved => 1,
            SlotState::Connected => 2,
        };
        data[2] = match self.model {
            Model::NotApplicable => 0,
            Model::PartialGyro => 1,
            Model::FullGyro => 2,
            Model::Unused => 3,
        };
        data[3] = match self.connection_type {
            ConnectionType::NotApplicable => 0,
            ConnectionType::Usb => 1,
            ConnectionType::Bluetooth => 2,
        };
        data[4..10].copy_from_slice(&self.mac);
        data[10] = match self.battery_status {
            BatteryStatus::NotApplicable => 0x00,
            BatteryStatus::Dying => 0x01,
            BatteryStatus::Low => 0x02,
            BatteryStatus::Medium => 0x03,
            BatteryStatus::High => 0x04,
            BatteryStatus::Full => 0x05,
            BatteryStatus::Charging => 0xEE,
            BatteryStatus::Charged => 0xEF,
        };
    }

    fn deserialize(data: &[u8; Self::LENGTH]) -> Result<Self, ControllerInfoError> {
        let slot = data[0];
        let slot_state = match data[1] {
            0 => SlotState::Disconnected,
            1 => SlotState::Reserved,
            2 => SlotState::Connected,
            slot_state => return Err(ControllerInfoError::InvalidSlotState(slot_state)),
        };
        let model = match data[2] {
            0 => Model::NotApplicable,
            1 => Model::PartialGyro,
            2 => Model::FullGyro,
            3 => Model::Unused,
            model => return Err(ControllerInfoError::InvalidModel(model)),
        };
        let connection_type = match data[3] {
            0 => ConnectionType::NotApplicable,
            1 => ConnectionType::Usb,
            2 => ConnectionType::Bluetooth,
            conn_type => return Err(ControllerInfoError::InvalidConnectionType(conn_type)),
        };
        let mac = data[4..10].try_into().unwrap();
        let battery_status = match data[10] {
            0x00 => BatteryStatus::NotApplicable,
            0x01 => BatteryStatus::Dying,
            0x02 => BatteryStatus::Low,
            0x03 => BatteryStatus::Medium,
            0x04 => BatteryStatus::High,
            0x05 => BatteryStatus::Full,
            0xEE => BatteryStatus::Charging,
            0xEF => BatteryStatus::Charged,
            bat => return Err(ControllerInfoError::InvalidBatteryStatus(bat)),
        };

        Ok(Self {
            slot,
            slot_state,
            model,
            connection_type,
            mac,
            battery_status,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RequestControllerInfoMessage {
    // invariant: 1 <= ports <= 4
    ports: usize,
    slots: [u8; 4],
}

impl RequestControllerInfoMessage {
    pub fn new1(slots: [u8; 1]) -> Self {
        Self {
            ports: 1,
            slots: [slots[0], 0, 0, 0],
        }
    }

    pub fn new2(slots: [u8; 2]) -> Self {
        Self {
            ports: 2,
            slots: [slots[0], slots[1], 0, 0],
        }
    }

    pub fn new3(slots: [u8; 3]) -> Self {
        Self {
            ports: 3,
            slots: [slots[0], slots[1], slots[2], 0],
        }
    }

    pub fn new4(slots: [u8; 4]) -> Self {
        Self { ports: 4, slots }
    }

    pub fn slots(&self) -> &[u8] {
        &self.slots[..self.ports]
    }

    pub fn len(&self) -> usize {
        self.ports + 4
    }

    /// Returns Err(()) if data.len() < self.len()
    pub fn serialize_to(&self, data: &mut [u8]) -> Result<(), BufferTooSmall> {
        if data.len() < self.len() {
            return Err(BufferTooSmall);
        }
        data[0..4].copy_from_slice(&(self.ports as i32).to_le_bytes());
        data[4..][..self.ports].copy_from_slice(&self.slots[..self.ports]);
        Ok(())
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, RequestControllerInfoError> {
        if data.len() < 4 {
            return Err(RequestControllerInfoError::NotEnoughData);
        }

        let ports = i32::from_le_bytes(data[0..4].try_into().unwrap());
        if ports < 1 || 4 < ports {
            return Err(RequestControllerInfoError::InvalidPortSize(ports));
        }
        let ports = ports as usize;

        if data.len() < 4 + ports {
            return Err(RequestControllerInfoError::NotEnoughData);
        }

        let mut slots = [0; 4];
        slots[..ports].copy_from_slice(&data[4..][..ports]);

        Ok(Self { ports, slots })
    }
}

#[derive(Clone, Debug)]
pub struct ControllerInfoMessage(pub ControllerInfo);

impl ControllerInfoMessage {
    pub const LENGTH: usize = 12;

    pub fn serialize(&self) -> [u8; Self::LENGTH] {
        let mut data = [0; Self::LENGTH];
        self.serialize_to(&mut data);
        data
    }

    pub fn serialize_to(&self, data: &mut [u8; Self::LENGTH]) {
        self.0
            .serialize_to(&mut data[..ControllerInfo::LENGTH].try_into().unwrap());
        data[11] = 0;
    }

    pub fn deserialize(data: &[u8; Self::LENGTH]) -> Result<Self, ControllerInfoError> {
        ControllerInfo::deserialize(&data[..ControllerInfo::LENGTH].try_into().unwrap()).map(Self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RegisterType {
    AllControllers,
    SlotBased(u8),
    MacBased([u8; 6]),
}

#[derive(Clone, Debug)]
pub struct RequestControllerDataMessage(pub RegisterType);

impl RequestControllerDataMessage {
    pub const LENGTH: usize = 8;

    pub fn serialize(&self) -> [u8; Self::LENGTH] {
        let mut data = [0; Self::LENGTH];
        self.serialize_to(&mut data);
        data
    }

    pub fn serialize_to(&self, data: &mut [u8; Self::LENGTH]) {
        match &self.0 {
            RegisterType::AllControllers => {
                data[0] = 0;
                data[1..8].copy_from_slice(&[0; 7]);
            }
            RegisterType::SlotBased(slot) => {
                data[0] = 1;
                data[1] = *slot;
                data[2..8].copy_from_slice(&[0; 6]);
            }
            RegisterType::MacBased(mac) => {
                data[0] = 2;
                data[1] = 0;
                data[2..8].copy_from_slice(mac);
            }
        }
    }

    pub fn deserialize(data: &[u8; Self::LENGTH]) -> Result<Self, RequestControllerDataError> {
        match data[0] {
            0 => Ok(RegisterType::AllControllers),
            1 if data[1] <= 4 => Ok(RegisterType::SlotBased(data[1])),
            1 => Err(RequestControllerDataError::InvalidSlot(data[1])),
            2 => Ok(RegisterType::MacBased(data[2..8].try_into().unwrap())),
            _ => Err(RequestControllerDataError::InvalidBitmask(data[0])),
        }
        .map(RequestControllerDataMessage)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Buttons([u8; 2]);

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
        let (index, bit) = rhs.index_and_bit();
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
    fn index_and_bit(&self) -> (usize, u8) {
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

#[derive(Clone, Debug)]
pub struct Motion {
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub gyro_pitch: f32,
    pub gyro_yaw: f32,
    pub gyro_roll: f32,
}

impl Motion {
    pub fn new() -> Self {
        Motion {
            accel_x: 0.,
            accel_y: 0.,
            accel_z: 0.,
            gyro_pitch: 0.,
            gyro_yaw: 0.,
            gyro_roll: 0.,
        }
    }

    fn as_array(&self) -> [f32; 6] {
        [
            self.accel_x,
            self.accel_y,
            self.accel_z,
            self.gyro_pitch,
            self.gyro_pitch,
            self.gyro_roll,
        ]
    }
}

#[derive(Clone, Debug)]
pub struct Touch {
    pub is_active: bool,
    pub id: u8,
    pub x: u16,
    pub y: u16,
}

impl Touch {
    const LENGTH: usize = 6;

    pub fn new(id: u8) -> Self {
        Touch {
            is_active: false,
            id: 0,
            x: 0,
            y: 0,
        }
    }

    fn serialize_to(&self, data: &mut [u8; Self::LENGTH]) {
        data[0] = if self.is_active { 1 } else { 0 };
        data[1] = self.id;
        data[2..4].copy_from_slice(&self.x.to_le_bytes());
        data[4..6].copy_from_slice(&self.y.to_le_bytes());
    }

    fn deserialize(data: &[u8; Self::LENGTH]) -> Self {
        Touch {
            is_active: data[0] != 0,
            id: data[1],
            x: u16::from_le_bytes([data[2], data[3]]),
            y: u16::from_le_bytes([data[4], data[5]]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ControllerDataMessage {
    pub info: ControllerInfo,
    pub connected: bool,
    pub packet_number: u32,
    pub buttons: Buttons,
    pub ps_button: u8,
    pub touch_button: u8,
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub analog_dpad_left: u8,
    pub analog_dpad_down: u8,
    pub analog_dpad_right: u8,
    pub analog_dpad_up: u8,
    pub analog_y: u8,
    pub analog_b: u8,
    pub analog_a: u8,
    pub analog_x: u8,
    pub analog_r1: u8,
    pub analog_l1: u8,
    pub analog_r2: u8,
    pub analog_l2: u8,
    pub first_touch: Touch,
    pub second_touch: Touch,
    pub motion_timestamp: u64,
    pub motion: Motion,
}

impl ControllerDataMessage {
    pub const LENGTH: usize = 80;
    
    pub fn new(info: ControllerInfo, connected: bool) -> Self {
        ControllerDataMessage {
            info,
            connected,
            packet_number: 0,
            buttons: Buttons::new(),
            ps_button: 0,
            touch_button: 0,
            left_stick_x: 0,
            left_stick_y: 0,
            right_stick_x: 0,
            right_stick_y: 0,
            analog_dpad_left: 0,
            analog_dpad_down: 0,
            analog_dpad_right: 0,
            analog_dpad_up: 0,
            analog_y: 0,
            analog_b: 0,
            analog_a: 0,
            analog_x: 0,
            analog_r1: 0,
            analog_l1: 0,
            analog_r2: 0,
            analog_l2: 0,
            first_touch: Touch::new(0),
            second_touch: Touch::new(0),
            motion_timestamp: 0,
            motion: Motion::new(),
        }
    }

    pub fn serialize(&self) -> [u8; Self::LENGTH] {
        let mut data = [0; Self::LENGTH];
        self.serialize_to(&mut data);
        data
    }

    pub fn serialize_to(&self, data: &mut [u8; Self::LENGTH]) {
        self.info.serialize_to(&mut data[0..11].try_into().unwrap());
        data[11] = if self.connected { 1 } else { 0 };
        data[12..16].copy_from_slice(&self.packet_number.to_le_bytes());
        data[16..18].copy_from_slice(&self.buttons.0);
        data[18] = self.ps_button;
        data[19] = self.touch_button;
        data[20] = self.left_stick_x;
        data[21] = self.left_stick_y;
        data[22] = self.right_stick_x;
        data[23] = self.right_stick_y;
        data[24] = self.analog_dpad_left;
        data[25] = self.analog_dpad_down;
        data[26] = self.analog_dpad_right;
        data[27] = self.analog_dpad_up;
        data[28] = self.analog_y;
        data[29] = self.analog_b;
        data[30] = self.analog_a;
        data[31] = self.analog_x;
        data[32] = self.analog_r1;
        data[33] = self.analog_l1;
        data[34] = self.analog_r2;
        data[35] = self.analog_l2;
        self.first_touch
            .serialize_to(&mut data[36..42].try_into().unwrap());
        self.second_touch
            .serialize_to(&mut data[42..48].try_into().unwrap());
        data[48..56].copy_from_slice(&self.motion_timestamp.to_le_bytes());
        for i in 0..6 {
            data[(56 + i * 4)..][..4].copy_from_slice(&self.motion.as_array()[i].to_le_bytes());
        }
    }

    pub fn deserialize(data: &[u8; Self::LENGTH]) -> Result<Self, ControllerInfoError> {
        let info =
            ControllerInfo::deserialize(data[0..ControllerInfo::LENGTH].try_into().unwrap())?;

        let mut motion = [0.0; 6];
        for i in 0..6 {
            motion[i] = f32::from_le_bytes(data[(56 + i * 4)..][..4].try_into().unwrap());
        }

        Ok(ControllerDataMessage {
            info,
            connected: data[11] != 0,
            packet_number: u32::from_le_bytes(data[12..16].try_into().unwrap()),
            buttons: Buttons([data[16], data[17]]),
            ps_button: data[18],
            touch_button: data[19],
            left_stick_x: data[20],
            left_stick_y: data[21],
            right_stick_x: data[22],
            right_stick_y: data[23],
            analog_dpad_left: data[24],
            analog_dpad_down: data[25],
            analog_dpad_right: data[26],
            analog_dpad_up: data[27],
            analog_y: data[28],
            analog_b: data[29],
            analog_a: data[30],
            analog_x: data[31],
            analog_r1: data[32],
            analog_l1: data[33],
            analog_r2: data[34],
            analog_l2: data[35],
            first_touch: Touch::deserialize(&data[36..42].try_into().unwrap()),
            second_touch: Touch::deserialize(&data[42..48].try_into().unwrap()),
            motion_timestamp: u64::from_le_bytes(data[48..56].try_into().unwrap()),
            motion: Motion {
                accel_x: motion[0],
                accel_y: motion[1],
                accel_z: motion[2],
                gyro_pitch: motion[3],
                gyro_yaw: motion[4],
                gyro_roll: motion[5],
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    header: MessageHeader,
    kind: MessageKind,
}

impl Message {
    /// The constructor will set the header length, crc, and type fields appropriately
    pub fn new(mut header: MessageHeader, kind: MessageKind) -> Self {
        header.message_type = kind.message_type();
        header.length = kind.len() as u16 + 4;
        header.crc32_hash = 0;

        Message { header, kind }
    }

    pub fn header(&self) -> &MessageHeader {
        &self.header
    }

    pub fn kind(&self) -> &MessageKind {
        &self.kind
    }

    pub fn serialize_to(&self, data: &mut [u8], crc_func: impl FnOnce(&[u8]) -> u32) -> Result<(), BufferTooSmall> {
        if self.len() > data.len() {
            return Err(BufferTooSmall);
        }
        self.header.serialize_to(&mut data[0..MessageHeader::LENGTH].try_into().unwrap());
        self.kind.serialize_to(&mut data[MessageHeader::LENGTH..])?;
        let crc32_hash = crc_func(&data[..self.len()]);
        data[8..12].copy_from_slice(&crc32_hash.to_le_bytes());
        
        Ok(())
    }

    /// Maximum len is 100 (BUFFER_SIZE)
    /// Minimum len is 20
    pub fn len(&self) -> usize {
        self.kind.len() + MessageHeader::LENGTH
    }
}

#[derive(Debug, Clone)]
pub enum MessageKind {
    RequestProtocolInfo(RequestProtocolInfoMessage),
    ProtocolInfo(ProtocolInfoMessage),
    RequestControllerInfo(RequestControllerInfoMessage),
    ControllerInfo(ControllerInfoMessage),
    RequestControllerData(RequestControllerDataMessage),
    ControllerData(ControllerDataMessage),
}

impl MessageKind {
    pub fn message_type(&self) -> MessageType {
        match self {
            MessageKind::RequestProtocolInfo(_) | MessageKind::ProtocolInfo(_) => {
                MessageType::ProtocolVersionInfo
            }
            MessageKind::RequestControllerInfo(_) | MessageKind::ControllerInfo(_) => {
                MessageType::ControllerInfo
            }
            MessageKind::RequestControllerData(_) | MessageKind::ControllerData(_) => {
                MessageType::ControllerData
            }
        }
    }

    /// Length of the message body (not including the message_type field)
    pub fn len(&self) -> usize {
        match self {
            MessageKind::RequestProtocolInfo(_) => RequestProtocolInfoMessage::LENGTH,
            MessageKind::ProtocolInfo(_) => ProtocolInfoMessage::LENGTH,
            MessageKind::RequestControllerInfo(msg) => msg.len(),
            MessageKind::ControllerInfo(_) => ControllerInfoMessage::LENGTH,
            MessageKind::RequestControllerData(_) => RequestControllerDataMessage::LENGTH,
            MessageKind::ControllerData(_) => ControllerDataMessage::LENGTH,
        }
    }

    fn serialize_to(&self, data: &mut [u8]) -> Result<(), BufferTooSmall> {
        if self.len() > data.len() {
            return Err(BufferTooSmall);
        }

        match self {
            MessageKind::RequestProtocolInfo(m) => {},
            MessageKind::ProtocolInfo(m) => m.serialize_to(&mut data[..ProtocolInfoMessage::LENGTH].try_into().unwrap()),
            MessageKind::RequestControllerInfo(m) => m.serialize_to(data)?,
            MessageKind::ControllerInfo(m) => m.serialize_to(&mut data[..ControllerInfoMessage::LENGTH].try_into().unwrap()),
            MessageKind::RequestControllerData(m) => m.serialize_to(&mut data[..RequestControllerDataMessage::LENGTH].try_into().unwrap()),
            MessageKind::ControllerData(m) => m.serialize_to(&mut data[..ControllerDataMessage::LENGTH].try_into().unwrap()),
        }

        Ok(())
    }
}
