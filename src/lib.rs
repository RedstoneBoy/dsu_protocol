pub mod error;
pub mod types;

use core::convert::TryInto;
use core::hash::Hasher;

use error::*;
use types::*;

pub const MAGIC_CLIENT: u32 = 0x43555344;
pub const MAGIC_SERVER: u32 = 0x53555344;

pub const MESSAGE_PROTOCOL: u32 = 0x100000;
pub const MESSAGE_INFO: u32 = 0x100001;
pub const MESSAGE_DATA: u32 = 0x100002;

pub trait Access<'a, const SIZE: usize> {
    type Buffer;
    type SizedBuffer;
}

pub struct Ref;
impl<'a, const SIZE: usize> Access<'a, SIZE> for Ref {
    type Buffer = &'a [u8];
    type SizedBuffer = &'a [u8; SIZE];
}

pub struct Mut;
impl<'a, const SIZE: usize> Access<'a, SIZE> for Mut {
    type Buffer = &'a mut [u8];
    type SizedBuffer = &'a mut [u8; SIZE];
}

pub struct Owned;
impl<'a, const SIZE: usize> Access<'a, SIZE> for Owned {
    type Buffer = [u8; SIZE];
    type SizedBuffer = [u8; SIZE];
}

macro_rules! buf_type {
    (message $name:ident, $size:literal) => {
        buf_type!($name, $size);

        impl<'a> $name<'a, Mut> {
            pub fn update_crc<H: Hasher>(&mut self, mut hasher: H) {
                hasher.write(&self.buf[0..8]);
                hasher.write(&[0u8; 4]);
                hasher.write(&self.buf[12..]);
                self.header_mut().set_crc32(hasher.finish() as u32);
            }
        }

        impl<'a> $name<'a, Owned> {
            fn new_uninit() -> Self {
                Self { buf: [0; $size] }
            }

            pub fn get(&self) -> $name<Ref> {
                $name::<Ref>::from_ref(&self.buf)
            }

            pub fn get_mut(&mut self) -> $name<Mut> {
                $name::<Mut>::from_mut(&mut self.buf)
            }
        }
    };
    ($name:ident, $size:literal) => {
        pub struct $name<'a, A: Access<'a, $size> = Owned> {
            buf: A::SizedBuffer,
        }

        impl<'a> $name<'a, Ref> {
            pub fn bytes(&self) -> &[u8] {
                self.buf
            }

            pub fn from_ref(buf: &'a [u8; $size]) -> Self {
                Self { buf }
            }
        }

        impl<'a> $name<'a, Mut> {
            pub fn bytes(&self) -> &[u8] {
                self.buf
            }

            pub fn bytes_mut(&mut self) -> &mut [u8] {
                self.buf
            }

            pub fn from_mut(buf: &'a mut [u8; $size]) -> Self {
                Self { buf }
            }
        }

        impl<'a> $name<'a, Owned> {
            pub fn bytes(&self) -> &[u8] {
                &self.buf
            }

            pub fn bytes_mut(&mut self) -> &mut [u8] {
                &mut self.buf
            }
        }
    };
    (dynamic message $name:ident, $size:literal) => {
        pub struct $name<'a, A: Access<'a, $size> = Owned> {
            buf: A::Buffer,
        }

        impl<'a> $name<'a, Owned> {
            fn new_uninit() -> Self {
                Self { buf: [0; $size] }
            }

            pub fn get(&self) -> $name<Ref> {
                $name::<Ref>::from_ref(&self.buf)
            }

            pub fn get_mut(&mut self) -> $name<Mut> {
                $name::<Mut>::from_mut(&mut self.buf)
            }
        }
    };
}

macro_rules! impl_new {
    ($name:ident, $($field:ident : $fieldty:ty),* $(,)?) => {
        impl<'a> $name<'a, Owned> {
            pub fn new<H: Hasher>(
                $($field: $fieldty,)*
                hasher: H
            ) -> Self {
                let mut this = Self::new_uninit();
                this.get_mut().initialize(
                    $($field,)*
                    hasher,
                );
                this
            }
        }
    };
}

macro_rules! int_fields {
    ($name:ident, $($field:ident $set_field:ident : $itype:ty = $range:expr),* $(,)?) => {
        impl<'a> $name<'a, Ref> {
            $(
                pub fn $field(&self) -> $itype {
                    <$itype>::from_le_bytes(self.buf[$range].try_into().unwrap())
                }
            )*
        }

        impl<'a> $name<'a, Mut> {
            $(
                pub fn $field(&self) -> $itype {
                    <$itype>::from_le_bytes(self.buf[$range].try_into().unwrap())
                }

                pub fn $set_field(&mut self, val: $itype) {
                    self.buf[$range].copy_from_slice(&val.to_le_bytes());
                }
            )*
        }
    };
}

macro_rules! enum_fields {
    ($name:ident, $($field:ident $set_field:ident from $valtype:ty [ $range:expr ] $enumtype:ty = $field_name:literal {
        $($enumraw:expr => $enumval:path,)* $(,)?
    })*) => {
        impl<'a> $name<'a, Ref> {
            $(
                pub fn $field(&self) -> Result<$enumtype, Invalid<$valtype>> {
                    match <$valtype>::from_le_bytes(self.buf[$range].try_into().unwrap()) {
                        $(val if val == $enumraw => Ok($enumval),)*
                        invalid => Err(Invalid(invalid, $field_name)),
                    }
                }
            )*
        }

        impl<'a> $name<'a, Mut> {
            $(
                pub fn $field(&self) -> Result<$enumtype, Invalid<$valtype>> {
                    match <$valtype>::from_le_bytes(self.buf[$range].try_into().unwrap()) {
                        $(val if val == $enumraw => Ok($enumval),)*
                        invalid => Err(Invalid(invalid, $field_name)),
                    }
                }

                pub fn $set_field(&mut self, val: $enumtype) {
                    let intval: $valtype = match val {
                        $($enumval => $enumraw,)*
                    };
                    self.buf[$range].copy_from_slice(&intval.to_le_bytes());
                }
            )*
        }
    };
}

macro_rules! sub_fields {
    ($name:ident, $($field:ident $field_mut:ident : $ftype:ident = $range:expr),* $(,)?) => {
        impl<'a> $name<'a, Ref> {
            $(
                pub fn $field(&self) -> $ftype<Ref> {
                    $ftype::<Ref>::from_ref(self.buf[$range].try_into().unwrap())
                }
            )*
        }

        impl<'a> $name<'a, Mut> {
            $(
                pub fn $field(&self) -> $ftype<Ref> {
                    $ftype::<Ref>::from_ref(self.buf[$range].try_into().unwrap())
                }

                pub fn $field_mut(&mut self) -> $ftype<Mut> {
                    $ftype::from_mut((&mut self.buf[$range]).try_into().unwrap())
                }
            )*
        }
    };
}

/*
pub struct Message<'a, A: Access<'a, 0>> {
    header: Header<'a, A>,
    body: Body<'a, A>,
}

impl<'a> Message<'a, Ref> {
    pub fn new(data: &'a [u8]) -> Self {
        todo!()
    }
}
*/

buf_type!(Header, 20);

int_fields!(Header,
    packet_length set_packet_length: u16 = 6..8,
    crc32         set_crc32:         u32 = 8..12,
    sender_id     set_sender_id:     u32 = 12..16,
);

enum_fields!(Header,
    magic set_magic from u32[0..4] Magic = "magic" {
        MAGIC_CLIENT => Magic::Client,
        MAGIC_SERVER => Magic::Server,
    }
    message_type set_message_type from u32[16..20] MessageType = "message_type" {
        MESSAGE_PROTOCOL => MessageType::ProtocolVersionInfo,
        MESSAGE_INFO     => MessageType::ControllerInfo,
        MESSAGE_DATA     => MessageType::ControllerData,
    }
    protocol set_protocol from u16[4..6] Protocol = "protocol" {
        1001 => Protocol::Version1001,
    }
);

impl<'a> Header<'a, Mut> {
    pub fn initialize(
        &mut self,
        magic: Magic,
        protocol: Protocol,
        length: u16,
        crc32: u32,
        sender_id: u32,
        message_type: MessageType,
    ) {
        self.set_magic(magic);
        self.set_protocol(protocol);
        self.set_packet_length(length);
        self.set_crc32(crc32);
        self.set_sender_id(sender_id);
        self.set_message_type(message_type);
    }
}

buf_type!(message RequestProtocolVersionInfo, 20);

sub_fields!(RequestProtocolVersionInfo,
    header header_mut: Header = 0..20,
);

impl<'a> RequestProtocolVersionInfo<'a, Mut> {
    pub fn initialize<H: Hasher>(&mut self, sender_id: u32, hasher: H) {
        self.header_mut().initialize(
            Magic::Client,
            Protocol::Version1001,
            20 - 16,
            0,
            sender_id,
            MessageType::ProtocolVersionInfo,
        );
        self.update_crc(hasher);
    }
}

impl_new!(RequestProtocolVersionInfo, sender_id: u32,);

buf_type!(message ProtocolVersionInfo, 22);

sub_fields!(ProtocolVersionInfo,
    header header_mut: Header = 0..20,
);

enum_fields!(ProtocolVersionInfo,
    protocol set_protocol from u16[(20 + 0)..2] Protocol = "protocol" {
        1001 => Protocol::Version1001,
    }
);

impl<'a> ProtocolVersionInfo<'a, Mut> {
    pub fn initialize<H: Hasher>(&mut self, sender_id: u32, protocol: Protocol, hasher: H) {
        self.header_mut().initialize(
            Magic::Server,
            protocol,
            22 - 16,
            0,
            sender_id,
            MessageType::ProtocolVersionInfo,
        );
        self.set_protocol(protocol);
        self.update_crc(hasher);
    }
}

impl_new!(ProtocolVersionInfo, sender_id: u32, protocol: Protocol,);

buf_type!(ControllerHeader, 11);

int_fields!(ControllerHeader,
    slot set_slot: u8 = 0..1,
);

enum_fields!(ControllerHeader,
    state set_state from u8[1..2] State = "state" {
        0 => State::Disconnected,
        1 => State::Reserved,
        2 => State::Connected,
    }
    model set_model from u8[2..3] Model = "model" {
        0 => Model::NotApplicable,
        1 => Model::PartialGyro,
        2 => Model::FullGyro,
        3 => Model::Unused,
    }
    connection_type set_connection_type from u8[3..4] ConnectionType = "connection_type" {
        0 => ConnectionType::NotApplicable,
        1 => ConnectionType::Usb,
        2 => ConnectionType::Bluetooth,
    }
    battery_status set_battery_status from u8[10..11] BatteryStatus = "battery_status" {
        0x00 => BatteryStatus::NotApplicable,
        0x01 => BatteryStatus::Dying,
        0x02 => BatteryStatus::Low,
        0x03 => BatteryStatus::Medium,
        0x04 => BatteryStatus::High,
        0x05 => BatteryStatus::Full,
        0xEE => BatteryStatus::Charging,
        0xEF => BatteryStatus::Charged,
    }
);

impl<'a> ControllerHeader<'a, Ref> {
    pub fn mac(&self) -> &[u8; 6] {
        self.buf[4..10].try_into().unwrap()
    }
}

impl<'a> ControllerHeader<'a, Mut> {
    pub fn initialize(
        &mut self,
        slot: u8,
        state: State,
        model: Model,
        connection_type: ConnectionType,
        mac: [u8; 6],
        battery_status: BatteryStatus,
    ) {
        self.set_slot(slot);
        self.set_state(state);
        self.set_model(model);
        self.set_connection_type(connection_type);
        *self.mac_mut() = mac;
        self.set_battery_status(battery_status);
    }

    pub fn mac(&self) -> &[u8; 6] {
        self.buf[4..10].try_into().unwrap()
    }

    pub fn mac_mut(&mut self) -> &mut [u8; 6] {
        (&mut self.buf[4..10]).try_into().unwrap()
    }
}

buf_type!(dynamic message RequestControllerInfo, 28);

sub_fields!(RequestControllerInfo,
    header header_mut: Header = 0..20,
);

impl<'a> RequestControllerInfo<'a, Ref> {
    pub fn from_ref(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    pub fn slots(&self) -> Result<&[u8], RequestControllerInfoError> {
        let port = self.num_slots()? as usize;
        if self.buf.len() < 20 + 4 + port {
            return Err(RequestControllerInfoError::SliceTooSmall);
        }
        Ok(&self.buf[24..][..port])
    }

    pub fn num_slots(&self) -> Result<usize, RequestControllerInfoError> {
        if self.buf.len() < 20 + 4 {
            return Err(RequestControllerInfoError::SliceTooSmall);
        }
        let port = i32::from_le_bytes(self.buf[20..24].try_into().unwrap());
        if port < 0 || 4 < port {
            return Err(RequestControllerInfoError::InvalidSlotsLength(port));
        }
        Ok(port as usize)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf[..28]
    }
}

impl<'a> RequestControllerInfo<'a, Mut> {
    pub fn from_mut(buf: &'a mut [u8]) -> Self {
        Self { buf }
    }

    pub fn initialize<H: Hasher>(
        &mut self,
        sender_id: u32,
        slots: &[u8],
        hasher: H,
    ) -> Result<(), RequestControllerInfoError> {
        let len = self.num_slots()? as u16;
        self.header_mut().initialize(
            Magic::Client,
            Protocol::Version1001,
            24 + len - 16,
            0,
            sender_id,
            MessageType::ProtocolVersionInfo,
        );
        self.set_slots(slots)?;
        self.update_crc(hasher)?;
        Ok(())
    }

    pub fn update_crc<H: Hasher>(
        &mut self,
        mut hasher: H,
    ) -> Result<(), RequestControllerInfoError> {
        let len = self.num_slots()?;
        hasher.write(&self.buf[0..8]);
        hasher.write(&[0u8; 4]);
        hasher.write(&self.buf[12..(24 + len)]);
        self.header_mut().set_crc32(hasher.finish() as u32);
        Ok(())
    }

    pub fn slots(&self) -> Result<&[u8], RequestControllerInfoError> {
        let port = self.num_slots()? as usize;
        if self.buf.len() < 20 + 4 + port {
            return Err(RequestControllerInfoError::SliceTooSmall);
        }
        Ok(&self.buf[24..][..port])
    }

    pub fn set_slots(&mut self, slots: &[u8]) -> Result<(), RequestControllerInfoError> {
        if slots.len() < 1 || 4 < slots.len() {
            return Err(RequestControllerInfoError::InvalidSlotsLength(
                slots.len() as u32 as i32,
            ));
        }
        if self.buf.len() < 20 + 4 + slots.len() {
            return Err(RequestControllerInfoError::SliceTooSmall);
        }
        self.buf[20..24].copy_from_slice(&(slots.len() as i32).to_le_bytes());
        self.buf[24..][..slots.len()].copy_from_slice(slots);
        Ok(())
    }

    pub fn num_slots(&self) -> Result<usize, RequestControllerInfoError> {
        if self.buf.len() < 20 + 4 {
            return Err(RequestControllerInfoError::SliceTooSmall);
        }
        let port = i32::from_le_bytes(self.buf[20..24].try_into().unwrap());
        if port < 0 || 4 < port {
            return Err(RequestControllerInfoError::InvalidSlotsLength(port));
        }
        Ok(port as usize)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf[..28]
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..28]
    }
}

impl<'a> RequestControllerInfo<'a, Owned> {
    pub fn new_owned<H: Hasher>(
        sender_id: u32,
        slots: &[u8],
        hasher: H,
    ) -> Result<Self, RequestControllerInfoError> {
        let mut this = Self::new_uninit();
        this.get_mut().initialize(sender_id, slots, hasher)?;
        Ok(this)
    }
}

buf_type!(message ControllerInfo, 32);

sub_fields!(ControllerInfo,
    header header_mut: Header = 0..20,
    controller_header controller_header_mut: ControllerHeader = 20..31,
);

impl<'a> ControllerInfo<'a, Mut> {
    pub fn initialize<H: Hasher>(
        &mut self,
        sender_id: u32,
        slot: u8,
        state: State,
        model: Model,
        connection_type: ConnectionType,
        mac: [u8; 6],
        battery_status: BatteryStatus,
        hasher: H,
    ) {
        self.header_mut().initialize(
            Magic::Server,
            Protocol::Version1001,
            32 - 16,
            0,
            sender_id,
            MessageType::ControllerInfo,
        );
        self.controller_header_mut().initialize(
            slot,
            state,
            model,
            connection_type,
            mac,
            battery_status,
        );
        self.update_crc(hasher);
    }
}

impl_new!(
    ControllerInfo,
    sender_id: u32,
    slot: u8,
    state: State,
    model: Model,
    connection_type: ConnectionType,
    mac: [u8; 6],
    battery_status: BatteryStatus,
);

buf_type!(message RequestControllerData, 28);

sub_fields!(RequestControllerData,
    header header_mut: Header = 0..20,
);

int_fields!(RequestControllerData,
    slot set_slot: u8 = 21..22,
);

enum_fields!(RequestControllerData,
    registration set_registration from u8[20..21] Registration = "registration" {
        0 => Registration::AllControllers,
        1 => Registration::SlotBased,
        2 => Registration::MacBased,
    }
);

impl<'a> RequestControllerData<'a, Ref> {
    pub fn mac(&self) -> &[u8; 6] {
        self.buf[22..28].try_into().unwrap()
    }
}

impl<'a> RequestControllerData<'a, Mut> {
    pub fn initialize<H: Hasher>(
        &mut self,
        sender_id: u32,
        registration: Registration,
        slot: u8,
        mac: [u8; 6],
        hasher: H,
    ) {
        self.header_mut().initialize(
            Magic::Client,
            Protocol::Version1001,
            28 - 16,
            0,
            sender_id,
            MessageType::ControllerData,
        );
        self.set_registration(registration);
        self.set_slot(slot);
        *self.mac_mut() = mac;
        self.update_crc(hasher);
    }

    pub fn mac(&self) -> &[u8; 6] {
        self.buf[22..28].try_into().unwrap()
    }

    pub fn mac_mut(&mut self) -> &mut [u8; 6] {
        (&mut self.buf[22..28]).try_into().unwrap()
    }
}

impl_new!(
    RequestControllerData,
    sender_id: u32,
    registration: Registration,
    slot: u8,
    mac: [u8; 6],
);

buf_type!(message ControllerData, 100);

sub_fields!(ControllerData,
    header header_mut: Header = 0..20,
    controller_header controller_header_mut: ControllerHeader = 20..31,
    touch1 touch1_mut: Touch = 56..62,
    touch2 touch2_mut: Touch = 62..68,
);

int_fields!(ControllerData,
    packet_number     set_packet_number:     u32 = 32..36,
    ps_button         set_ps_button:         u8  = 38..39,
    touch_button      set_touch_button:      u8  = 39..40,
    left_stick_x      set_left_stick_x:      u8  = 40..41,
    left_stick_y      set_left_stick_y:      u8  = 41..42,
    right_stick_x     set_right_stick_x:     u8  = 42..43,
    right_stick_y     set_right_stick_y:     u8  = 43..44,
    analog_dpad_left  set_analog_dpad_left:  u8  = 44..45,
    analog_dpad_down  set_analog_dpad_down:  u8  = 45..46,
    analog_dpad_right set_analog_dpad_right: u8  = 46..47,
    analog_dpad_up    set_analog_dpad_up:    u8  = 47..48,
    analog_y          set_analog_y:          u8  = 48..49,
    analog_b          set_analog_b:          u8  = 49..50,
    analog_a          set_analog_a:          u8  = 50..51,
    analog_x          set_analog_x:          u8  = 51..52,
    analog_r1         set_analog_r1:         u8  = 52..53,
    analog_l1         set_analog_l1:         u8  = 53..54,
    analog_r2         set_analog_r2:         u8  = 54..55,
    analog_l2         set_analog_l2:         u8  = 55..56,
    motion_timestamp  set_motion_timestamp:  u64 = 68..76,
    accel_x           set_accel_x:           f32 = 76..80,
    accel_y           set_accel_y:           f32 = 80..84,
    accel_z           set_accel_z:           f32 = 84..88,
    gyro_pitch        set_gyro_pitch:        f32 = 88..92,
    gyro_yaw          set_gyro_yaw:          f32 = 92..96,
    gyro_roll         set_gyro_roll:         f32 = 96..100,
);

impl<'a> ControllerData<'a, Ref> {
    pub fn is_connected(&self) -> bool {
        self.buf[31] != 0
    }

    pub fn buttons(&self) -> Buttons {
        Buttons(self.buf[36..38].try_into().unwrap())
    }
}

impl<'a> ControllerData<'a, Mut> {
    pub fn initialize<H: Hasher>(
        &mut self,
        sender_id: u32,
        slot: u8,
        state: State,
        model: Model,
        connection_type: ConnectionType,
        mac: [u8; 6],
        battery_status: BatteryStatus,
        connected: bool,
        hasher: H,
    ) {
        self.header_mut().initialize(
            Magic::Server,
            Protocol::Version1001,
            100 - 16,
            0,
            sender_id,
            MessageType::ControllerData,
        );
        self.controller_header_mut().initialize(
            slot,
            state,
            model,
            connection_type,
            mac,
            battery_status,
        );
        self.set_connected(connected);
        self.update_crc(hasher);
    }

    pub fn is_connected(&self) -> bool {
        self.buf[31] != 0
    }

    pub fn set_connected(&mut self, val: bool) {
        self.buf[31] = if val { 1 } else { 0 };
    }

    pub fn buttons(&self) -> Buttons {
        Buttons(self.buf[36..38].try_into().unwrap())
    }

    pub fn set_buttons(&mut self, buttons: Buttons) {
        self.buf[36] = buttons.0[0];
        self.buf[37] = buttons.0[1];
    }

    pub fn clear_analog_buttons(&mut self) {
        self.set_analog_dpad_left(0);
        self.set_analog_dpad_down(0);
        self.set_analog_dpad_right(0);
        self.set_analog_dpad_up(0);
        self.set_analog_y(0);
        self.set_analog_b(0);
        self.set_analog_a(0);
        self.set_analog_x(0);
        self.set_analog_r1(0);
        self.set_analog_l1(0);
        self.set_analog_r2(0);
        self.set_analog_l2(0);
    }
}

impl_new!(
    ControllerData,
    sender_id: u32,
    slot: u8,
    state: State,
    model: Model,
    connection_type: ConnectionType,
    mac: [u8; 6],
    battery_status: BatteryStatus,
    connected: bool,
);

buf_type!(Touch, 6);

int_fields!(Touch,
    touch_id set_touch_id: u8 = 1..2,
    touch_x  set_touch_x:  u8 = 2..4,
    touch_y  set_touch_y:  u8 = 4..6,
);

impl<'a> Touch<'a, Ref> {
    pub fn is_active(&self) -> bool {
        self.buf[0] != 0
    }
}

impl<'a> Touch<'a, Mut> {
    pub fn is_active(&self) -> bool {
        self.buf[0] != 0
    }

    pub fn set_active(&mut self, val: bool) {
        self.buf[0] = if val { 1 } else { 0 };
    }
}

pub enum MessageRef<'a> {
    RequestProtocolVersionInfo(RequestProtocolVersionInfo<'a, Ref>),
    ProtocolVersionInfo(ProtocolVersionInfo<'a, Ref>),
    RequestControllerInfo(RequestControllerInfo<'a, Ref>),
    ControllerInfo(ControllerInfo<'a, Ref>),
    RequestControllerData(RequestControllerData<'a, Ref>),
    ControllerData(ControllerData<'a, Ref>),
}

impl<'a> MessageRef<'a> {
    pub fn parse<H: Hasher>(buf: &'a [u8], mut hasher: H) -> Result<Self, MessageParseError> {
        let header = Header::from_ref(
            buf[0..20].try_into()
                .map_err(|_| MessageParseError::SliceTooSmall)?,
        );
        let magic = header
            .magic()
            .map_err(|Invalid(magic, _)| MessageParseError::InvalidMagic(magic))?;
        let message_type = header
            .message_type()
            .map_err(|Invalid(id, _)| MessageParseError::InvalidMessageId(id))?;

        let this = match (magic, message_type) {
            (Magic::Client, MessageType::ProtocolVersionInfo) => {
                Self::RequestProtocolVersionInfo(RequestProtocolVersionInfo::from_ref(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
            (Magic::Server, MessageType::ProtocolVersionInfo) => {
                Self::ProtocolVersionInfo(ProtocolVersionInfo::from_ref(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
            (Magic::Client, MessageType::ControllerInfo) => {
                Self::RequestControllerInfo(RequestControllerInfo::from_ref(buf))
            }
            (Magic::Server, MessageType::ControllerInfo) => {
                Self::ControllerInfo(ControllerInfo::from_ref(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
            (Magic::Client, MessageType::ControllerData) => {
                Self::RequestControllerData(RequestControllerData::from_ref(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
            (Magic::Server, MessageType::ControllerData) => {
                Self::ControllerData(ControllerData::from_ref(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
        };

        let bytes = match &this {
            Self::RequestProtocolVersionInfo(v) => v.bytes(),
            Self::ProtocolVersionInfo(v) => v.bytes(),
            Self::RequestControllerInfo(v) => v.bytes(),
            Self::ControllerInfo(v) => v.bytes(),
            Self::RequestControllerData(v) => v.bytes(),
            Self::ControllerData(v) => v.bytes(),
        };
        hasher.write(&bytes[0..8]);
        hasher.write(&[0u8; 4]);
        hasher.write(&bytes[12..]);
        let calc_hash = hasher.finish() as u32;
        let hash = this.header().crc32();
        if hash != calc_hash {
            return Err(MessageParseError::InvalidCrc32 {
                expected: hash,
                calculated: calc_hash,
            });
        }

        Ok(this)
    }

    pub fn header(&self) -> Header<Ref> {
        match self {
            Self::RequestProtocolVersionInfo(v) => v.header(),
            Self::ProtocolVersionInfo(v) => v.header(),
            Self::RequestControllerInfo(v) => v.header(),
            Self::ControllerInfo(v) => v.header(),
            Self::RequestControllerData(v) => v.header(),
            Self::ControllerData(v) => v.header(),
        }
    }
}

pub enum MessageMut<'a> {
    RequestProtocolVersionInfo(RequestProtocolVersionInfo<'a, Mut>),
    ProtocolVersionInfo(ProtocolVersionInfo<'a, Mut>),
    RequestControllerInfo(RequestControllerInfo<'a, Mut>),
    ControllerInfo(ControllerInfo<'a, Mut>),
    RequestControllerData(RequestControllerData<'a, Mut>),
    ControllerData(ControllerData<'a, Mut>),
}

impl<'a> MessageMut<'a> {
    pub fn parse_mut<H: Hasher>(
        buf: &'a mut [u8],
        mut hasher: H,
    ) -> Result<Self, MessageParseError> {
        let header = Header::from_mut(
            (&mut buf[0..20]).try_into()
                .map_err(|_| MessageParseError::SliceTooSmall)?,
        );
        let magic = header
            .magic()
            .map_err(|Invalid(magic, _)| MessageParseError::InvalidMagic(magic))?;
        let message_type = header
            .message_type()
            .map_err(|Invalid(id, _)| MessageParseError::InvalidMessageId(id))?;

        let this = match (magic, message_type) {
            (Magic::Client, MessageType::ProtocolVersionInfo) => {
                Self::RequestProtocolVersionInfo(RequestProtocolVersionInfo::from_mut(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
            (Magic::Server, MessageType::ProtocolVersionInfo) => {
                Self::ProtocolVersionInfo(ProtocolVersionInfo::from_mut(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
            (Magic::Client, MessageType::ControllerInfo) => {
                Self::RequestControllerInfo(RequestControllerInfo::from_mut(buf))
            }
            (Magic::Server, MessageType::ControllerInfo) => {
                Self::ControllerInfo(ControllerInfo::from_mut(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
            (Magic::Client, MessageType::ControllerData) => {
                Self::RequestControllerData(RequestControllerData::from_mut(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
            (Magic::Server, MessageType::ControllerData) => {
                Self::ControllerData(ControllerData::from_mut(
                    buf.try_into()
                        .map_err(|_| MessageParseError::SliceTooSmall)?,
                ))
            }
        };

        let bytes = match &this {
            Self::RequestProtocolVersionInfo(v) => v.bytes(),
            Self::ProtocolVersionInfo(v) => v.bytes(),
            Self::RequestControllerInfo(v) => v.bytes(),
            Self::ControllerInfo(v) => v.bytes(),
            Self::RequestControllerData(v) => v.bytes(),
            Self::ControllerData(v) => v.bytes(),
        };
        hasher.write(&bytes[0..8]);
        hasher.write(&[0u8; 4]);
        hasher.write(&bytes[12..]);
        let calc_hash = hasher.finish() as u32;
        let hash = this.header().crc32();
        if hash != calc_hash {
            return Err(MessageParseError::InvalidCrc32 {
                expected: hash,
                calculated: calc_hash,
            });
        }

        Ok(this)
    }

    pub fn header(&self) -> Header<Ref> {
        match self {
            Self::RequestProtocolVersionInfo(v) => v.header(),
            Self::ProtocolVersionInfo(v) => v.header(),
            Self::RequestControllerInfo(v) => v.header(),
            Self::ControllerInfo(v) => v.header(),
            Self::RequestControllerData(v) => v.header(),
            Self::ControllerData(v) => v.header(),
        }
    }

    pub fn header_mut(&mut self) -> Header<Mut> {
        match self {
            Self::RequestProtocolVersionInfo(v) => v.header_mut(),
            Self::ProtocolVersionInfo(v) => v.header_mut(),
            Self::RequestControllerInfo(v) => v.header_mut(),
            Self::ControllerInfo(v) => v.header_mut(),
            Self::RequestControllerData(v) => v.header_mut(),
            Self::ControllerData(v) => v.header_mut(),
        }
    }
}
