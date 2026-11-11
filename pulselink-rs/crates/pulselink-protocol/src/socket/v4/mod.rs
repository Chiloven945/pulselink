//! DG-LAB V4 WebSocket protocol types.

pub mod device;
pub mod event;
pub mod frame;
pub mod operation;
pub mod rpc;

pub use device::{DevicePatch, RemoteDevice, SlotPatch, SlotPatchEntry};
pub use event::AppEvent;
pub use frame::RelayFrame;
pub use operation::{
    ActionType, ClearOperate, DeviceOperate, DeviceOperateResult, OperateValue, OperationEndReason,
    PulseFrameWire,
};
pub use rpc::{AppMessage, RpcRequest, RpcResponse};
