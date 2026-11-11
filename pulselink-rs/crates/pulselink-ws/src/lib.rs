//! DG-LAB V4 WebSocket controller SDK.
//!
//! The crate owns the controller-side WebSocket connection, RPC requests,
//! multi-app registry, device cache, and `device.op` submission. It does not
//! depend on BLE or the local-device runtime.

mod app;
mod device;
mod error;
mod operation;
mod options;
mod registry;
mod rpc;
mod socket;
mod transport;

#[cfg(feature = "mock")]
pub mod testing;

pub use app::{AppClient, WsEvent};
pub use device::{RemoteDeviceHandle, RemoteDeviceSnapshot};
pub use error::WsError;
pub use operation::{
    ClearTarget, OperationHandle, OperationOptions, OperationResult, PendingOperation, PulseFrame,
};
pub use options::SocketOptions;
pub use socket::{Socket, SocketEvent, SocketState};

pub use pulselink_core::{Channel, Intensity};
pub use pulselink_protocol::socket::v4::{
    ActionType, AppEvent, AppMessage, ClearOperate, DeviceOperate, DeviceOperateResult,
    DevicePatch, OperateValue, OperationEndReason, PulseFrameWire, RelayFrame, RemoteDevice,
    RpcRequest, RpcResponse, SlotPatch, SlotPatchEntry,
};
