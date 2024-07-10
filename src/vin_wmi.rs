pub fn vin_wmi(vin: &str) -> String {
    let wmi = if vin.len() > 3 { &vin[..3] } else { vin };
    if &wmi[2..3] == "9" && vin.len() >= 14 {
        wmi.to_owned() + &vin[11..14]
    } else {
        wmi.to_owned()
    }
}
