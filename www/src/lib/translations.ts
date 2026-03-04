export const appStoreScreenshots = {
  en: {
    simplicity: "System Stats\nin Your Menu Bar",
    performance: "So Light\nYou Won't Notice It",
    privacy: "Runs Locally\nNo Telemetry",
  },
  es: {
    simplicity: "Estadísticas del Sistema\nen Tu Barra de Menú",
    performance: "Tan Ligero\nQue No Lo Notarás",
    privacy: "Funciona Localmente\nSin Telemetría",
  },
  fr: {
    simplicity: "Statistiques Système\ndans Votre Barre de Menu",
    performance: "Si Léger\nQu'on Ne Le Remarque Pas",
    privacy: "Fonctionne Localement\nSans Télémétrie",
  },
  de: {
    simplicity: "Systemstatistiken\nin Ihrer Menüleiste",
    performance: "So Leicht\nSie Merken Es Kaum",
    privacy: "Läuft Lokal\nKeine Telemetrie",
  },
  ja: {
    simplicity: "システム統計を\nメニューバーに表示",
    performance: "気づかないほど\n軽い動作",
    privacy: "ローカル実行\nテレメトリなし",
  },
  "pt-BR": {
    simplicity: "Estatísticas do Sistema\nna Sua Barra de Menu",
    performance: "Tão Leve\nQue Você Nem Nota",
    privacy: "Funciona Localmente\nSem Telemetria",
  },
  "zh-Hans": {
    simplicity: "系统状态\n显示在菜单栏",
    performance: "轻盈运行\n几乎无感",
    privacy: "本地运行\n无遥测数据",
  },
} as const;

export type Lang = keyof typeof appStoreScreenshots;
export type ScreenshotKey = keyof (typeof appStoreScreenshots)["en"];

export const supportedLangs = Object.keys(appStoreScreenshots) as Lang[];
export const screenshotKeys: ScreenshotKey[] = ["simplicity", "performance", "privacy"];
