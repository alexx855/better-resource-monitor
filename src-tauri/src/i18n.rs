pub struct Translations {
    pub start_at_login: &'static str,
    pub show_memory: &'static str,
    pub show_cpu: &'static str,
    pub show_storage: &'static str,
    pub show_gpu: &'static str,
    pub show_network: &'static str,
    pub show_thermal_status: &'static str,
    pub show_alert_colors: &'static str,
    pub quit: &'static str,
    pub system_monitor: &'static str,
    pub thermal_status: &'static str,
    pub thermal_nominal: &'static str,
    pub thermal_fair: &'static str,
    pub thermal_serious: &'static str,
    pub thermal_critical: &'static str,
    pub thermal_nominal_explanation: &'static str,
    pub thermal_fair_explanation: &'static str,
    pub thermal_serious_explanation: &'static str,
    pub thermal_critical_explanation: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    show_storage: "Show Storage",
    show_gpu: "Show GPU",
    show_network: "Show Network",
    show_thermal_status: "Show Thermal Status",
    show_alert_colors: "Show Alert Colors",
    quit: "Quit",
    system_monitor: "System Monitor",
    thermal_status: "Thermal Status",
    thermal_nominal: "Nominal",
    thermal_fair: "Fair",
    thermal_serious: "Serious",
    thermal_critical: "Critical",
    thermal_nominal_explanation: "Thermal conditions are normal.",
    thermal_fair_explanation: "Thermal pressure is elevated.",
    thermal_serious_explanation: "Thermal pressure is high.",
    thermal_critical_explanation: "Thermal pressure is critical.",
};

const SPANISH: Translations = Translations {
    start_at_login: "Abrir al iniciar sesión",
    show_memory: "Mostrar memoria",
    show_cpu: "Mostrar CPU",
    show_storage: "Mostrar almacenamiento",
    show_gpu: "Mostrar GPU",
    show_network: "Mostrar uso de red",
    show_thermal_status: "Mostrar estado térmico",
    show_alert_colors: "Mostrar colores de alerta",
    quit: "Salir",
    system_monitor: "Monitor del sistema",
    thermal_status: "Estado térmico",
    thermal_nominal: "Nominal",
    thermal_fair: "Moderado",
    thermal_serious: "Grave",
    thermal_critical: "Crítico",
    thermal_nominal_explanation: "Las condiciones térmicas son normales.",
    thermal_fair_explanation: "La presión térmica es moderada.",
    thermal_serious_explanation: "La presión térmica es alta.",
    thermal_critical_explanation: "La presión térmica es crítica.",
};

const PORTUGUESE: Translations = Translations {
    start_at_login: "Iniciar com o sistema",
    show_memory: "Mostrar memória",
    show_cpu: "Mostrar CPU",
    show_storage: "Mostrar armazenamento",
    show_gpu: "Mostrar GPU",
    show_network: "Mostrar rede",
    show_thermal_status: "Mostrar status térmico",
    show_alert_colors: "Mostrar alertas de cor",
    quit: "Sair",
    system_monitor: "Monitor do sistema",
    thermal_status: "Status térmico",
    thermal_nominal: "Nominal",
    thermal_fair: "Moderado",
    thermal_serious: "Grave",
    thermal_critical: "Crítico",
    thermal_nominal_explanation: "As condições térmicas estão normais.",
    thermal_fair_explanation: "A pressão térmica está moderada.",
    thermal_serious_explanation: "A pressão térmica está alta.",
    thermal_critical_explanation: "A pressão térmica está crítica.",
};

const CHINESE: Translations = Translations {
    start_at_login: "登录时启动",
    show_memory: "显示内存",
    show_cpu: "显示 CPU",
    show_storage: "显示存储空间",
    show_gpu: "显示 GPU",
    show_network: "显示网络",
    show_thermal_status: "显示热状态",
    show_alert_colors: "显示警报颜色",
    quit: "退出",
    system_monitor: "系统监控器",
    thermal_status: "热状态",
    thermal_nominal: "正常",
    thermal_fair: "一般",
    thermal_serious: "严重",
    thermal_critical: "危急",
    thermal_nominal_explanation: "热状况正常。",
    thermal_fair_explanation: "热压力有所升高。",
    thermal_serious_explanation: "热压力较高。",
    thermal_critical_explanation: "热压力达到危急水平。",
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
