use std::collections::HashMap;
use std::collections::HashSet;

pub struct AvailableOSs {
    // TODO this may not ever be needed.
    #[allow(dead_code)]
    oss_by_name: HashMap<String, AvailableOS>,
}

#[derive(Clone, Debug)]
pub struct AvailableOS {
    pub name: String,
    pub desktop: bool,
    pub family: Option<String>,
}

impl Default for AvailableOSs {
    fn default() -> Self {
        let available_oss = available_operating_systems();
        let desktop_oss = desktop_oss();
        let os_families = os_families()
            .into_iter()
            .flat_map(|(k, shorts)| {
                let k = &k;
                shorts
                    .into_iter()
                    .map(move |short| (short, k.clone()))
                    .collect::<Vec<(String, String)>>()
            })
            .collect::<HashMap<String, String>>();

        let oss_by_short = available_oss
            .into_iter()
            .map(|(short, os)| {
                let family = os_families.get(&short).cloned();

                let desktop = if let Some(family) = &family {
                    desktop_oss.contains(family)
                } else {
                    false
                };

                (
                    short,
                    AvailableOS {
                        name: os,
                        desktop,
                        family,
                    },
                )
            })
            .collect::<HashMap<String, AvailableOS>>();

        let oss_by_name = oss_by_short
            .values()
            .map(|os| (os.name.replace(' ', "").to_lowercase(), os.clone()))
            .collect::<HashMap<String, AvailableOS>>();

        Self { oss_by_name }
    }
}

impl AvailableOSs {
    pub fn search_by_name(&self, name: &str) -> Option<&AvailableOS> {
        let name = name.to_lowercase().replace(' ', "");
        self.oss_by_name.get(&name)
    }
}

fn available_operating_systems() -> HashMap<String, String> {
    [
        ("AIX", "AIX"),
        ("AND", "Android"),
        ("ADR", "Android TV"),
        ("ALP", "Alpine Linux"),
        ("AMZ", "Amazon Linux"),
        ("AMG", "AmigaOS"),
        ("ARM", "Armadillo OS"),
        ("ARO", "AROS"),
        ("ATV", "tvOS"),
        ("ARL", "Arch Linux"),
        ("AOS", "AOSC OS"),
        ("ASP", "ASPLinux"),
        ("BTR", "BackTrack"),
        ("SBA", "Bada"),
        ("BEO", "BeOS"),
        ("BYI", "Baidu Yi"),
        ("BLB", "BlackBerry OS"),
        ("QNX", "BlackBerry Tablet OS"),
        ("BOS", "Bliss OS"),
        ("BMP", "Brew"),
        ("BSN", "BrightSignOS"),
        ("CAI", "Caixa Mágica"),
        ("CES", "CentOS"),
        ("CST", "CentOS Stream"),
        ("CLO", "Clear Linux OS"),
        ("CLR", "ClearOS Mobile"),
        ("COS", "Chrome OS"),
        ("CRS", "Chromium OS"),
        ("CHN", "China OS"),
        ("CYN", "CyanogenMod"),
        ("DEB", "Debian"),
        ("DEE", "Deepin"),
        ("DFB", "DragonFly"),
        ("DVK", "DVKBuntu"),
        ("ELE", "ElectroBSD"),
        ("EUL", "EulerOS"),
        ("FED", "Fedora"),
        ("FEN", "Fenix"),
        ("FOS", "Firefox OS"),
        ("FIR", "Fire OS"),
        ("FOR", "Foresight Linux"),
        ("FRE", "Freebox"),
        ("BSD", "FreeBSD"),
        ("FRI", "FRITZ!OS"),
        ("FYD", "FydeOS"),
        ("FUC", "Fuchsia"),
        ("GNT", "Gentoo"),
        ("GNX", "GENIX"),
        ("GEO", "GEOS"),
        ("GNS", "gNewSense"),
        ("GRI", "GridOS"),
        ("GTV", "Google TV"),
        ("HPX", "HP-UX"),
        ("HAI", "Haiku OS"),
        ("IPA", "iPadOS"),
        ("HAR", "HarmonyOS"),
        ("HAS", "HasCodingOS"),
        ("HEL", "HELIX OS"),
        ("IRI", "IRIX"),
        ("INF", "Inferno"),
        ("JME", "Java ME"),
        ("JOL", "Joli OS"),
        ("KOS", "KaiOS"),
        ("KAL", "Kali"),
        ("KAN", "Kanotix"),
        ("KIN", "KIN OS"),
        ("KNO", "Knoppix"),
        ("KTV", "KreaTV"),
        ("KBT", "Kubuntu"),
        ("LIN", "GNU/Linux"),
        ("LND", "LindowsOS"),
        ("LNS", "Linspire"),
        ("LEN", "Lineage OS"),
        ("LIR", "Liri OS"),
        ("LOO", "Loongnix"),
        ("LBT", "Lubuntu"),
        ("LOS", "Lumin OS"),
        ("LUN", "LuneOS"),
        ("VLN", "VectorLinux"),
        ("MAC", "Mac"),
        ("MAE", "Maemo"),
        ("MAG", "Mageia"),
        ("MDR", "Mandriva"),
        ("SMG", "MeeGo"),
        ("MCD", "MocorDroid"),
        ("MON", "moonOS"),
        ("EZX", "Motorola EZX"),
        ("MIN", "Mint"),
        ("MLD", "MildWild"),
        ("MOR", "MorphOS"),
        ("NBS", "NetBSD"),
        ("MTK", "MTK / Nucleus"),
        ("MRE", "MRE"),
        ("NXT", "NeXTSTEP"),
        ("NWS", "NEWS-OS"),
        ("WII", "Nintendo"),
        ("NDS", "Nintendo Mobile"),
        ("NOV", "Nova"),
        ("OS2", "OS/2"),
        ("T64", "OSF1"),
        ("OBS", "OpenBSD"),
        ("OVS", "OpenVMS"),
        ("OVZ", "OpenVZ"),
        ("OWR", "OpenWrt"),
        ("OTV", "Opera TV"),
        ("ORA", "Oracle Linux"),
        ("ORD", "Ordissimo"),
        ("PAR", "Pardus"),
        ("PCL", "PCLinuxOS"),
        ("PIC", "PICO OS"),
        ("PLA", "Plasma Mobile"),
        ("PSP", "PlayStation Portable"),
        ("PS3", "PlayStation"),
        ("PVE", "Proxmox VE"),
        ("PUR", "PureOS"),
        ("QTP", "Qtopia"),
        ("PIO", "Raspberry Pi OS"),
        ("RAS", "Raspbian"),
        ("RHT", "Red Hat"),
        ("RST", "Red Star"),
        ("RED", "RedOS"),
        ("REV", "Revenge OS"),
        ("ROS", "RISC OS"),
        ("ROC", "Rocky Linux"),
        ("ROK", "Roku OS"),
        ("RSO", "Rosa"),
        ("ROU", "RouterOS"),
        ("REM", "Remix OS"),
        ("RRS", "Resurrection Remix OS"),
        ("REX", "REX"),
        ("RZD", "RazoDroiD"),
        ("SAB", "Sabayon"),
        ("SSE", "SUSE"),
        ("SAF", "Sailfish OS"),
        ("SCI", "Scientific Linux"),
        ("SEE", "SeewoOS"),
        ("SER", "SerenityOS"),
        ("SIR", "Sirin OS"),
        ("SLW", "Slackware"),
        ("SOS", "Solaris"),
        ("SBL", "Star-Blade OS"),
        ("SYL", "Syllable"),
        ("SYM", "Symbian"),
        ("SYS", "Symbian OS"),
        ("S40", "Symbian OS Series 40"),
        ("S60", "Symbian OS Series 60"),
        ("SY3", "Symbian^3"),
        ("TEN", "TencentOS"),
        ("TDX", "ThreadX"),
        ("TIZ", "Tizen"),
        ("TIV", "TiVo OS"),
        ("TOS", "TmaxOS"),
        ("TUR", "Turbolinux"),
        ("UBT", "Ubuntu"),
        ("ULT", "ULTRIX"),
        ("UOS", "UOS"),
        ("VID", "VIDAA"),
        ("WAS", "watchOS"),
        ("WER", "Wear OS"),
        ("WTV", "WebTV"),
        ("WHS", "Whale OS"),
        ("WIN", "Windows"),
        ("WCE", "Windows CE"),
        ("WIO", "Windows IoT"),
        ("WMO", "Windows Mobile"),
        ("WPH", "Windows Phone"),
        ("WRT", "Windows RT"),
        ("WPO", "WoPhone"),
        ("XBX", "Xbox"),
        ("XBT", "Xubuntu"),
        ("YNS", "YunOS"),
        ("ZEN", "Zenwalk"),
        ("ZOR", "ZorinOS"),
        ("IOS", "iOS"),
        ("POS", "palmOS"),
        ("WEB", "Webian"),
        ("WOS", "webOS"),
    ]
    .into_iter()
    .map(|(short, name)| (short.to_owned(), name.to_owned()))
    .collect::<HashMap<String, String>>()
}

fn os_families() -> HashMap<String, Vec<String>> {
    [
        (
            "Android",
            vec![
                "AND", "CYN", "FIR", "REM", "RZD", "MLD", "MCD", "YNS", "GRI", "HAR", "ADR", "CLR",
                "BOS", "REV", "LEN", "SIR", "RRS", "WER", "PIC", "ARM", "HEL", "BYI",
            ],
        ),
        ("AmigaOS", vec!["AMG", "MOR", "ARO"]),
        ("BlackBerry", vec!["BLB", "QNX"]),
        ("Brew", vec!["BMP"]),
        ("BeOS", vec!["BEO", "HAI"]),
        ("Chrome OS", vec!["COS", "CRS", "FYD", "SEE"]),
        ("Firefox OS", vec!["FOS", "KOS"]),
        ("Gaming Console", vec!["WII", "PS3"]),
        ("Google TV", vec!["GTV"]),
        ("IBM", vec!["OS2"]),
        ("iOS", vec!["IOS", "ATV", "WAS", "IPA"]),
        ("RISC OS", vec!["ROS"]),
        (
            "GNU/Linux",
            vec![
                "LIN", "ARL", "DEB", "KNO", "MIN", "UBT", "KBT", "XBT", "LBT", "FED", "RHT", "VLN",
                "MDR", "GNT", "SAB", "SLW", "SSE", "CES", "BTR", "SAF", "ORD", "TOS", "RSO", "DEE",
                "FRE", "MAG", "FEN", "CAI", "PCL", "HAS", "LOS", "DVK", "ROK", "OWR", "OTV", "KTV",
                "PUR", "PLA", "FUC", "PAR", "FOR", "MON", "KAN", "ZEN", "LND", "LNS", "CHN", "AMZ",
                "TEN", "CST", "NOV", "ROU", "ZOR", "RED", "KAL", "ORA", "VID", "TIV", "BSN", "RAS",
                "UOS", "PIO", "FRI", "LIR", "WEB", "SER", "ASP", "AOS", "LOO", "EUL", "SCI", "ALP",
                "CLO", "ROC", "OVZ", "PVE", "RST", "EZX", "GNS", "JOL", "TUR", "QTP", "WPO",
            ],
        ),
        ("Mac", vec!["MAC"]),
        ("Mobile Gaming Console", vec!["PSP", "NDS", "XBX"]),
        ("OpenVMS", vec!["OVS"]),
        ("Real-time OS", vec!["MTK", "TDX", "MRE", "JME", "REX"]),
        (
            "Other Mobile",
            vec!["WOS", "POS", "SBA", "TIZ", "SMG", "MAE", "LUN", "GEO"],
        ),
        ("Symbian", vec!["SYM", "SYS", "SY3", "S60", "S40"]),
        (
            "Unix",
            vec![
                "SOS", "AIX", "HPX", "BSD", "NBS", "OBS", "DFB", "SYL", "IRI", "T64", "INF", "ELE",
                "GNX", "ULT", "NWS", "NXT", "SBL",
            ],
        ),
        ("WebTV", vec!["WTV"]),
        ("Windows", vec!["WIN"]),
        (
            "Windows Mobile",
            vec!["WPH", "WMO", "WCE", "WRT", "WIO", "KIN"],
        ),
        ("Other Smart TV", vec!["WHS"]),
    ]
    .into_iter()
    .map(|(brand, families)| {
        (
            brand.to_owned(),
            families
                .into_iter()
                .map(|f| f.to_owned())
                .collect::<Vec<String>>(),
        )
    })
    .collect::<HashMap<String, Vec<String>>>()
}

pub fn desktop_oss() -> HashSet<String> {
    [
        "AmigaOS",
        "IBM",
        "GNU/Linux",
        "Mac",
        "Unix",
        "Windows",
        "BeOS",
        "Chrome OS",
        "Chromium OS",
    ]
    .into_iter()
    .map(|f| f.to_owned())
    .collect::<HashSet<String>>()
}
