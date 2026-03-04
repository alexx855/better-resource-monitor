pub struct Translations {
    pub start_at_login: &'static str,
    pub show_memory: &'static str,
    pub show_cpu: &'static str,
    pub show_gpu: &'static str,
    pub show_network: &'static str,
    pub show_alert_colors: &'static str,
    pub quit: &'static str,
    pub system_monitor: &'static str,
}

#[derive(Clone, Copy)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Japanese,
    Portuguese,
    Chinese,
}

const ENGLISH: Translations = Translations {
    start_at_login: "Start at Login",
    show_memory: "Show Memory",
    show_cpu: "Show CPU",
    show_gpu: "Show GPU",
    show_network: "Show Network",
    show_alert_colors: "Show Alert Colors",
    quit: "Quit",
    system_monitor: "System Monitor",
};

const SPANISH: Translations = Translations {
    start_at_login: "Iniciar con el sistema",
    show_memory: "Mostrar memoria",
    show_cpu: "Mostrar CPU",
    show_gpu: "Mostrar GPU",
    show_network: "Mostrar red",
    show_alert_colors: "Mostrar alertas de color",
    quit: "Salir",
    system_monitor: "Monitor del sistema",
};

const FRENCH: Translations = Translations {
    start_at_login: "Lancer au démarrage",
    show_memory: "Afficher la mémoire",
    show_cpu: "Afficher le CPU",
    show_gpu: "Afficher le GPU",
    show_network: "Afficher le réseau",
    show_alert_colors: "Afficher les alertes couleur",
    quit: "Quitter",
    system_monitor: "Moniteur système",
};

const GERMAN: Translations = Translations {
    start_at_login: "Beim Anmelden starten",
    show_memory: "Speicher anzeigen",
    show_cpu: "CPU anzeigen",
    show_gpu: "GPU anzeigen",
    show_network: "Netzwerk anzeigen",
    show_alert_colors: "Farbwarnungen anzeigen",
    quit: "Beenden",
    system_monitor: "Systemmonitor",
};

const JAPANESE: Translations = Translations {
    start_at_login: "ログイン時に開始",
    show_memory: "メモリを表示",
    show_cpu: "CPUを表示",
    show_gpu: "GPUを表示",
    show_network: "ネットワークを表示",
    show_alert_colors: "アラート色を表示",
    quit: "終了",
    system_monitor: "システムモニター",
};

const PORTUGUESE: Translations = Translations {
    start_at_login: "Iniciar com o sistema",
    show_memory: "Mostrar memória",
    show_cpu: "Mostrar CPU",
    show_gpu: "Mostrar GPU",
    show_network: "Mostrar rede",
    show_alert_colors: "Mostrar alertas de cor",
    quit: "Sair",
    system_monitor: "Monitor do sistema",
};

const CHINESE: Translations = Translations {
    start_at_login: "登录时启动",
    show_memory: "显示内存",
    show_cpu: "显示 CPU",
    show_gpu: "显示 GPU",
    show_network: "显示网络",
    show_alert_colors: "显示警报颜色",
    quit: "退出",
    system_monitor: "系统监控器",
};

impl Language {
    pub fn translations(self) -> &'static Translations {
        match self {
            Language::English => &ENGLISH,
            Language::Spanish => &SPANISH,
            Language::French => &FRENCH,
            Language::German => &GERMAN,
            Language::Japanese => &JAPANESE,
            Language::Portuguese => &PORTUGUESE,
            Language::Chinese => &CHINESE,
        }
    }
}

pub fn detect_language() -> Language {
    let locale = sys_locale::get_locale().unwrap_or_default();
    let prefix = locale.split(['-', '_']).next().unwrap_or("en");

    match prefix {
        "es" => Language::Spanish,
        "fr" => Language::French,
        "de" => Language::German,
        "ja" => Language::Japanese,
        "pt" => Language::Portuguese,
        "zh" => Language::Chinese,
        _ => Language::English,
    }
}
