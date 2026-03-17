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
        "pt" => Language::Portuguese,
        "zh" => Language::Chinese,
        _ => Language::English,
    }
}
