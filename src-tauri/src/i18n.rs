pub struct Translations {
    pub start_at_login: &'static str,
    pub show_memory: &'static str,
    pub show_cpu: &'static str,
    pub show_storage: &'static str,
    pub show_gpu: &'static str,
    pub show_network: &'static str,
    pub show_alert_colors: &'static str,
    pub quit: &'static str,
    pub system_monitor: &'static str,
    pub thermal_status: &'static str,
    pub thermal_unavailable: &'static str,
    pub thermal_nominal: &'static str,
    pub thermal_fair: &'static str,
    pub thermal_serious: &'static str,
    pub thermal_critical: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    English,
    Spanish,
    Portuguese,
    Chinese,
}

const ENGLISH: Translations = Translations {
    start_at_login: "Start at login",
    show_memory: "Show memory",
    show_cpu: "Show CPU",
    show_storage: "Show storage",
    show_gpu: "Show GPU",
    show_network: "Show network",
    show_alert_colors: "Show warning colors",
    quit: "Quit",
    system_monitor: "System monitor",
    thermal_status: "Thermal status",
    thermal_unavailable: "unavailable",
    thermal_nominal: "nominal",
    thermal_fair: "fair",
    thermal_serious: "serious",
    thermal_critical: "critical",
};

const SPANISH: Translations = Translations {
    start_at_login: "Abrir al iniciar sesión",
    show_memory: "Mostrar memoria",
    show_cpu: "Mostrar CPU",
    show_storage: "Mostrar almacenamiento",
    show_gpu: "Mostrar GPU",
    show_network: "Mostrar uso de red",
    show_alert_colors: "Mostrar colores de advertencia",
    quit: "Salir",
    system_monitor: "Monitor del sistema",
    thermal_status: "Estado térmico",
    thermal_unavailable: "no disponible",
    thermal_nominal: "normal",
    thermal_fair: "elevado",
    thermal_serious: "alto",
    thermal_critical: "crítico",
};

const PORTUGUESE: Translations = Translations {
    start_at_login: "Iniciar com o sistema",
    show_memory: "Mostrar memória",
    show_cpu: "Mostrar CPU",
    show_storage: "Mostrar armazenamento",
    show_gpu: "Mostrar GPU",
    show_network: "Mostrar rede",
    show_alert_colors: "Mostrar cores de aviso",
    quit: "Sair",
    system_monitor: "Monitor do sistema",
    thermal_status: "Estado térmico",
    thermal_unavailable: "indisponível",
    thermal_nominal: "normal",
    thermal_fair: "elevado",
    thermal_serious: "alto",
    thermal_critical: "crítico",
};

const CHINESE: Translations = Translations {
    start_at_login: "登录时启动",
    show_memory: "显示内存",
    show_cpu: "显示 CPU",
    show_storage: "显示存储空间",
    show_gpu: "显示 GPU",
    show_network: "显示网络",
    show_alert_colors: "显示警告颜色",
    quit: "退出",
    system_monitor: "系统监控器",
    thermal_status: "温度状态",
    thermal_unavailable: "不可用",
    thermal_nominal: "正常",
    thermal_fair: "偏高",
    thermal_serious: "高",
    thermal_critical: "危急",
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

fn language_for_locale(locale: &str) -> Option<Language> {
    let prefix = locale.split(['-', '_']).next()?.to_ascii_lowercase();

    match prefix.as_str() {
        "en" => Some(Language::English),
        "es" => Some(Language::Spanish),
        "pt" => Some(Language::Portuguese),
        "zh" => Some(Language::Chinese),
        _ => None,
    }
}

pub(crate) fn detect_language_from_locales<I, S>(locales: I) -> Language
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    locales
        .into_iter()
        .find_map(|locale| language_for_locale(locale.as_ref()))
        .unwrap_or(Language::English)
}

pub fn detect_language() -> Language {
    detect_language_from_locales(sys_locale::get_locales())
}
