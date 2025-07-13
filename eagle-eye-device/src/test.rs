use std::str::FromStr;

use crate::Device;

#[test]
fn test_display_device() {
    let device = Device::default().id(0).key([0; 32]).user_name("").os("");
    let got = device.to_string();
    let want = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_owned();
    assert_eq!(got, want);

    let device = Device::default()
        .id(7654321)
        .key([69; 32])
        .user_name("new-user")
        .os("linux");
    let got = device.to_string();
    let want = "0000000000000000000000000074cbb14545454545454545454545454545454545454545454545454545454545454545086e65772d75736572056c696e7578".to_owned();
    assert_eq!(got, want);
}

#[test]
fn test_device_from_str() {
    let input = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_owned();
    let got = Device::from_str(&input).unwrap();
    let want = Device::default().id(0).key([0; 32]).user_name("").os("");
    assert_eq!(got, want);

    let input = "0000000000000000000000000074cbb14545454545454545454545454545454545454545454545454545454545454545086e65772d75736572056c696e7578".to_owned();
    let got = Device::from_str(&input).unwrap();
    let want = Device::default()
        .id(7654321)
        .key([69; 32])
        .user_name("new-user")
        .os("linux");
    assert_eq!(got, want);
}
