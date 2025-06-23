use eagle_eye_proto::server::EagleEyeServerSync;

struct EagleEyeDaemon<T> {
    id: [u8; 32],
    name: String,
    server: EagleEyeServerSync<T>,
}
