import type { FaqItem } from "./faq-types";
import type { ComparisonPageKey, SupportedLocale } from "./site";

export const embeddedFaqHeadings = {
  home: {
    en: "Common questions",
    es: "Preguntas comunes",
    "pt-br": "Perguntas comuns",
    "zh-cn": "常见问题",
  },
  comparisonIndex: {
    en: "Choosing a menu bar monitor",
    es: "Cómo elegir un monitor para la barra de menús",
    "pt-br": "Como escolher um monitor de barra de menus",
    "zh-cn": "如何选择菜单栏监视器",
  },
  legal: {
    en: "Product privacy questions",
    es: "Preguntas de privacidad del producto",
    "pt-br": "Perguntas de privacidade do produto",
    "zh-cn": "产品隐私问题",
  },
} satisfies Record<string, Record<SupportedLocale, string>>;

export const comparisonFaqHeadings = {
  en: "Comparison questions",
  es: "Preguntas de comparación",
  "pt-br": "Perguntas sobre a comparação",
  "zh-cn": "对比问题",
} satisfies Record<SupportedLocale, string>;

const sharedFaqIds = {
  home: [
    "overview",
    "macos-support",
    "pricing",
    "resource-use",
    "gpu-monitoring",
    "privacy",
    "battery",
    "activity-monitor",
  ],
  comparisonIndex: ["overview", "resource-use", "gpu-monitoring", "privacy", "comparison", "battery"],
  comparisonPage: ["overview", "resource-use", "gpu-monitoring", "privacy", "battery"],
  legal: ["privacy"],
} as const;

export function pickFaqItems(items: FaqItem[], key: keyof typeof sharedFaqIds) {
  const itemById = new Map(items.map((item) => [item.id, item]));
  return sharedFaqIds[key].map((id) => {
    const item = itemById.get(id);
    if (!item) {
      throw new Error(`Missing FAQ item with id "${id}"`);
    }
    return item;
  });
}

const comparisonOrder: ComparisonPageKey[] = ["vs-stats", "vs-istat-menus", "vs-eul"];

export function getComparisonIndexFaqItems(locale: SupportedLocale): FaqItem[] {
  const copy = {
    en: [
      {
        question: "Which monitor should I choose if I only want the basics?",
        answer:
          "Choose Better Resource Monitor if you want CPU, memory, GPU, and network visible in the menu bar without fan, thermal, or sensor depth.",
      },
      {
        question: "When should I compare individual apps?",
        answer:
          "Use the comparison pages when you are deciding between specific tools: [Stats]({{vsStatsUrl}}), [iStat Menus]({{vsIstatMenusUrl}}), or [Eul]({{vsEulUrl}}).",
      },
    ],
    es: [
      {
        question: "¿Qué monitor debería elegir si solo quiero lo básico?",
        answer:
          "Elige Better Resource Monitor si quieres ver CPU, memoria, GPU y red en la barra de menús sin profundizar en ventiladores, temperatura ni sensores.",
      },
      {
        question: "¿Cuándo conviene comparar apps concretas?",
        answer:
          "Usa las páginas de comparación cuando estés decidiendo entre herramientas específicas: [Stats]({{vsStatsUrl}}), [iStat Menus]({{vsIstatMenusUrl}}) o [Eul]({{vsEulUrl}}).",
      },
    ],
    "pt-br": [
      {
        question: "Qual monitor devo escolher se só quero o básico?",
        answer:
          "Escolha o Better Resource Monitor se você quer CPU, memória, GPU e rede visíveis na barra de menus sem detalhes de ventoinha, temperatura ou sensores.",
      },
      {
        question: "Quando devo comparar aplicativos específicos?",
        answer:
          "Use as páginas de comparação quando estiver decidindo entre ferramentas específicas: [Stats]({{vsStatsUrl}}), [iStat Menus]({{vsIstatMenusUrl}}) ou [Eul]({{vsEulUrl}}).",
      },
    ],
    "zh-cn": [
      {
        question: "如果我只需要基础功能，应该选择哪个监视器？",
        answer:
          "如果你只想在菜单栏查看 CPU、内存、GPU 和网络，而不需要风扇、温度或深层传感器信息，请选择 Better Resource Monitor。",
      },
      {
        question: "什么时候需要查看具体应用的对比？",
        answer:
          "当你正在具体比较工具时，可以查看这些页面：[Stats]({{vsStatsUrl}})、[iStat Menus]({{vsIstatMenusUrl}}) 或 [Eul]({{vsEulUrl}})。",
      },
    ],
  } satisfies Record<SupportedLocale, FaqItem[]>;

  return copy[locale];
}

export function getComparisonFaqItems(pageKey: ComparisonPageKey, locale: SupportedLocale): FaqItem[] {
  const copy = {
    "vs-stats": {
      en: [
        {
          question: "Is Better Resource Monitor a good Stats alternative?",
          answer:
            "Yes, if you want a Stats alternative focused on CPU, memory, GPU, and network. Stats is better if you need fan, thermal, and deeper sensor detail.",
        },
        {
          question: "Why does Stats need more system access?",
          answer:
            "Stats goes deeper into fan and thermal readings, which can require a helper. Better Resource Monitor avoids that category of data and stays sandbox-friendly.",
        },
      ],
      es: [
        {
          question: "¿Better Resource Monitor es una buena alternativa a Stats?",
          answer:
            "Sí, si quieres una alternativa a Stats más simple, centrada en CPU, memoria, GPU y red. Stats es más amplio si necesitas ventiladores, temperaturas y sensores más profundos.",
        },
        {
          question: "¿Por qué Stats necesita más acceso al sistema?",
          answer:
            "Stats profundiza en lecturas de ventiladores y temperatura, lo que puede requerir un helper. Better Resource Monitor evita esa categoría de datos y se mantiene compatible con sandbox.",
        },
      ],
      "pt-br": [
        {
          question: "O Better Resource Monitor é uma boa alternativa ao Stats?",
          answer:
            "Sim, se você quer uma alternativa ao Stats mais simples, focada em CPU, memória, GPU e rede. O Stats é mais amplo se você precisa de ventoinhas, temperatura e sensores mais detalhados.",
        },
        {
          question: "Por que o Stats precisa de mais acesso ao sistema?",
          answer:
            "O Stats aprofunda leituras de ventoinhas e temperatura, o que pode exigir um helper. O Better Resource Monitor evita essa categoria de dados e permanece compatível com sandbox.",
        },
      ],
      "zh-cn": [
        {
          question: "Better Resource Monitor 是好的 Stats 替代品吗？",
          answer:
            "是的，如果你想要一个更简单、专注于 CPU、内存、GPU 和网络的 Stats 替代品。若需要风扇、温度和更深层传感器，Stats 覆盖更广。",
        },
        {
          question: "为什么 Stats 需要更多系统访问权限？",
          answer:
            "Stats 会读取更深层的风扇和温度数据，因此可能需要辅助工具。Better Resource Monitor 避开这类数据，因此更适合沙盒环境。",
        },
      ],
    },
    "vs-istat-menus": {
      en: [
        {
          question: "Is Better Resource Monitor an iStat Menus replacement?",
          answer:
            "It can replace iStat Menus if your main goal is a lighter menu bar monitor. If you need a feature-rich dashboard with lots of sensors and controls, iStat Menus is usually the better fit.",
        },
        {
          question: "Why choose Better Resource Monitor instead of iStat Menus?",
          answer:
            "Choose Better Resource Monitor if you care most about low overhead, free pricing, open source code, and Mac App Store availability.",
        },
      ],
      es: [
        {
          question: "¿Better Resource Monitor puede reemplazar a iStat Menus?",
          answer:
            "Puede reemplazar a iStat Menus para una monitorización ligera en la barra de menús. iStat Menus sigue siendo mejor si quieres un panel de sensores más completo.",
        },
        {
          question: "¿Por qué elegir Better Resource Monitor en lugar de iStat Menus?",
          answer:
            "Elige Better Resource Monitor si priorizas bajo consumo, precio gratuito, código abierto y disponibilidad en la Mac App Store.",
        },
      ],
      "pt-br": [
        {
          question: "O Better Resource Monitor substitui o iStat Menus?",
          answer:
            "Ele pode substituir o iStat Menus para monitoramento leve na barra de menus. O iStat Menus continua melhor se você quer um painel mais completo de sensores.",
        },
        {
          question: "Por que escolher o Better Resource Monitor em vez do iStat Menus?",
          answer:
            "Escolha o Better Resource Monitor se você prioriza baixo consumo, preço gratuito, código aberto e disponibilidade na Mac App Store.",
        },
      ],
      "zh-cn": [
        {
          question: "Better Resource Monitor 能替代 iStat Menus 吗？",
          answer:
            "如果你需要轻量级菜单栏监控，它可以替代 iStat Menus。如果你想要覆盖更广的高级传感器仪表盘，iStat Menus 仍然更合适。",
        },
        {
          question: "为什么选择 Better Resource Monitor 而不是 iStat Menus？",
          answer:
            "如果你更看重低资源占用、免费、开源代码以及 Mac App Store 可用性，请选择 Better Resource Monitor。",
        },
      ],
    },
    "vs-eul": {
      en: [
        {
          question: "How is Better Resource Monitor different from Eul?",
          answer:
            "Both are lightweight options. Better Resource Monitor is narrower, actively packaged for the Mac App Store, and focuses on CPU, memory, GPU, and network.",
        },
        {
          question: "Which one is better for daily menu bar monitoring?",
          answer:
            "Better Resource Monitor is the simpler choice if you want a maintained Mac App Store app with the core numbers visible all day.",
        },
      ],
      es: [
        {
          question: "¿En qué se diferencia Better Resource Monitor de Eul?",
          answer:
            "Ambas son opciones ligeras. Better Resource Monitor es más enfocado, está empaquetado activamente para la Mac App Store y se centra en CPU, memoria, GPU y red.",
        },
        {
          question: "¿Cuál es mejor para la monitorización diaria en la barra de menús?",
          answer:
            "Better Resource Monitor es la opción más simple si quieres una app mantenida en la Mac App Store con las métricas principales visibles todo el día.",
        },
      ],
      "pt-br": [
        {
          question: "Como o Better Resource Monitor é diferente do Eul?",
          answer:
            "Ambos são opções leves. O Better Resource Monitor é mais focado, tem empacotamento ativo para a Mac App Store e se concentra em CPU, memória, GPU e rede.",
        },
        {
          question: "Qual é melhor para monitoramento diário na barra de menus?",
          answer:
            "O Better Resource Monitor é a escolha mais simples se você quer um app mantido na Mac App Store com os números principais visíveis o dia todo.",
        },
      ],
      "zh-cn": [
        {
          question: "Better Resource Monitor 和 Eul 有什么不同？",
          answer:
            "两者都是轻量级选择。Better Resource Monitor 更聚焦，持续为 Mac App Store 打包，并专注于 CPU、内存、GPU 和网络。",
        },
        {
          question: "哪一个更适合日常菜单栏监控？",
          answer:
            "如果你想要一个持续维护、可在 Mac App Store 获取，并且全天显示核心指标的应用，Better Resource Monitor 是更简单的选择。",
        },
      ],
    },
  } satisfies Record<ComparisonPageKey, Record<SupportedLocale, FaqItem[]>>;

  return comparisonOrder.includes(pageKey) ? copy[pageKey][locale] : [];
}
