pub mod ping_client_factory;
pub mod ping_client;
mod ping_client_tcp;

// quinn cannot be built for windows.arm64, because it doesn't support uint128 and cause compile
// failure in boringssl and ring. So before it is ready, we will have to ignore it.
#[cfg(any(not(target_os = "windows"), not(target_arch = "aarch64")))]
mod ping_client_quic;

#[cfg(test)]
mod ping_client_test_common;
