// Database of known controllers for the launcher.

#[derive(Clone, Debug)]
pub struct ControllerEntry {
    pub name: &'static str,
    pub vendor: &'static str,
    pub kind: &'static str,
}

/// URL/file-safe id for a controller image.
/// Must stay in sync with `tools/fetch-controller-images.ps1` (Get-Slug).
pub fn slug(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_sep = false;
        } else if !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

pub fn controller_database() -> Vec<ControllerEntry> {
    let mut v = Vec::new();
    macro_rules! add {
        ($name:expr, $vendor:expr, $kind:expr) => { v.push(ControllerEntry { name: $name, vendor: $vendor, kind: $kind }); };
    }
    // FPV radio transmitters (listed first — the usual choice for drone pilots)
    add!("Radiomaster TX16S MkII", "Radiomaster", "Radio");
    add!("Radiomaster TX16S", "Radiomaster", "Radio");
    add!("Radiomaster Boxer", "Radiomaster", "Radio");
    add!("Radiomaster Zorro", "Radiomaster", "Radio");
    add!("Radiomaster Pocket", "Radiomaster", "Radio");
    add!("Radiomaster T8 Lite", "Radiomaster", "Radio");
    add!("FrSky Taranis X9D Plus", "FrSky", "Radio");
    add!("FrSky Taranis Q X7", "FrSky", "Radio");
    add!("FrSky Taranis X-Lite S", "FrSky", "Radio");
    add!("Jumper T-Pro V2", "Jumper", "Radio");
    add!("Jumper T20", "Jumper", "Radio");
    add!("Jumper T15", "Jumper", "Radio");
    add!("Jumper T16", "Jumper", "Radio");
    add!("BetaFPV LiteRadio 3 SE", "BetaFPV", "Radio");
    add!("BetaFPV LiteRadio 2 Pro", "BetaFPV", "Radio");
    add!("FlySky FS-i6X", "FlySky", "Radio");
    add!("FlySky FS-i6S", "FlySky", "Radio");
    add!("FlySky Nirvana NV14", "FlySky", "Radio");
    add!("Spektrum DX6e", "Spektrum", "Radio");
    add!("Spektrum NX8", "Spektrum", "Radio");
    // Xbox
    add!("Xbox Wireless Controller", "Microsoft", "Gamepad");
    add!("Xbox Elite Wireless Controller Series 2", "Microsoft", "Gamepad");
    add!("Xbox 360 Controller", "Microsoft", "Gamepad");
    add!("Xbox One Controller", "Microsoft", "Gamepad");
    // PlayStation
    add!("DualShock 4", "Sony", "Gamepad");
    add!("DualSense", "Sony", "Gamepad");
    add!("DualSense Edge", "Sony", "Gamepad");
    // Nintendo
    add!("Nintendo Switch Pro Controller", "Nintendo", "Gamepad");
    add!("Joy-Con (L)", "Nintendo", "Gamepad");
    add!("Joy-Con (R)", "Nintendo", "Gamepad");
    // Logitech
    add!("Logitech F310", "Logitech", "Gamepad");
    add!("Logitech F710", "Logitech", "Gamepad");
    add!("Logitech Extreme 3D Pro", "Logitech", "Joystick");
    add!("Logitech G940", "Logitech", "HOTAS");
    add!("Logitech G Saitek X52", "Logitech", "HOTAS");
    add!("Logitech G Saitek X56", "Logitech", "HOTAS");
    // Thrustmaster
    add!("Thrustmaster T.16000M", "Thrustmaster", "Joystick");
    add!("Thrustmaster HOTAS Warthog", "Thrustmaster", "HOTAS");
    add!("Thrustmaster T.Flight HOTAS X", "Thrustmaster", "HOTAS");
    add!("Thrustmaster T.Flight HOTAS 4", "Thrustmaster", "HOTAS");
    add!("Thrustmaster Airbus Sidestick", "Thrustmaster", "Joystick");
    add!("Thrustmaster TWCS Throttle", "Thrustmaster", "Throttle");
    // VKB
    add!("VKB Gladiator NXT", "VKB", "Joystick");
    add!("VKB Gunfighter Mk.III", "VKB", "Joystick");
    add!("VKB STECS Throttle", "VKB", "Throttle");
    // Virpil
    add!("Virpil Alpha Prime", "Virpil", "Joystick");
    add!("Virpil MongoosT-50CM3", "Virpil", "Joystick");
    add!("Virpil CM3 Throttle", "Virpil", "Throttle");
    // Winwing
    add!("Winwing Orion 2", "Winwing", "Joystick");
    add!("Winwing Orion HOTAS", "Winwing", "HOTAS");
    add!("Winwing F-16EX Throttle", "Winwing", "Throttle");
    // Saitek
    add!("Saitek X52 Pro", "Saitek", "HOTAS");
    add!("Saitek Pro Flight Yoke", "Saitek", "Yoke");
    add!("Saitek Pro Flight Rudder Pedals", "Saitek", "Rudder");
    // Mad Catz
    add!("Mad Catz F.L.Y.5", "Mad Catz", "Joystick");
    // Razer
    add!("Razer Wolverine V2", "Razer", "Gamepad");
    add!("Razer Kishi", "Razer", "Gamepad");
    // 8BitDo
    add!("8BitDo SN30 Pro", "8BitDo", "Gamepad");
    add!("8BitDo Pro 2", "8BitDo", "Gamepad");
    add!("8BitDo Ultimate", "8BitDo", "Gamepad");
    // Generic
    add!("Generic USB Joystick", "Generic", "Joystick");
    add!("Generic Gamepad", "Generic", "Gamepad");
    add!("Generic HOTAS", "Generic", "HOTAS");
    // More to reach 60+
    add!("HORI Fighting Commander", "HORI", "Gamepad");
    add!("HORI Split Pad Pro", "HORI", "Gamepad");
    add!("PowerA Enhanced Controller", "PowerA", "Gamepad");
    add!("SCUF Instinct", "SCUF", "Gamepad");
    add!("Nacon Revolution 5 Pro", "Nacon", "Gamepad");
    add!("Steam Deck Controller", "Valve", "Gamepad");
    add!("Valve Index Controller", "Valve", "VR");
    add!("Oculus Touch", "Meta", "VR");
    add!("Meta Quest Touch", "Meta", "VR");
    add!("CH Products Fighterstick", "CH Products", "Joystick");
    add!("CH Products Pro Throttle", "CH Products", "Throttle");
    add!("CH Products Pro Pedals", "CH Products", "Rudder");
    add!("Realsimulator FSSB R3", "Realsimulator", "Joystick");
    add!("Realsimulator R2", "Realsimulator", "Rudder");
    add!("Honeycomb Alpha Yoke", "Honeycomb", "Yoke");
    add!("Honeycomb Bravo Throttle", "Honeycomb", "Throttle");
    add!("Turtle Beach Recon", "Turtle Beach", "Gamepad");
    add!("Victrix Pro BFG", "Victrix", "Gamepad");
    add!("PDP Afterglow", "PDP", "Gamepad");
    add!("SteelSeries Stratus+", "SteelSeries", "Gamepad");
    v
}
