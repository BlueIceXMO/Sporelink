export type ThemeTokens = {
  bg: string;
  bgElevated: string;
  surface: string;
  surfaceHover: string;
  card: string;
  cardFloating: string;
  button: string;
  userMessage: string;
  quotation: string;
  border: string;
  borderStrong: string;
  text: string;
  textSecondary: string;
  textTertiary: string;
  brand: string;
  brandSoft: string;
  brandText: string;
  danger: string;
  warning: string;
  success: string;
  shadow: string;
  glow: string;
  overlay: string;
  codeBg: string;
};

export type Theme = {
  id: string;
  name: string;
  nameZh: string;
  colorScheme: "dark" | "light";
  tokens: ThemeTokens;
};

const darkBase = {
  overlay: "rgba(0,0,0,.56)",
  shadow: "0 24px 70px rgba(0,0,0,.46)",
  glow: "0 0 0 1px rgba(255,255,255,.03), 0 18px 50px rgba(0,0,0,.32)",
};

export const THEMES: Theme[] = [
  {
    id: "ink",
    name: "Ink",
    nameZh: "墨",
    colorScheme: "dark",
    tokens: {
      ...darkBase,
      bg: "#0b0d0c",
      bgElevated: "#101411",
      surface: "#151a16",
      surfaceHover: "#1d241e",
      card: "#1b211c",
      cardFloating: "#252d26",
      button: "#303a31",
      userMessage: "#3b473c",
      quotation: "#263028",
      border: "#334034",
      borderStrong: "#48594a",
      text: "#f3f7f2",
      textSecondary: "#c7d2c7",
      textTertiary: "#8f9c90",
      brand: "#8ee39a",
      brandSoft: "rgba(142,227,154,.14)",
      brandText: "#06120a",
      danger: "#ff7d7d",
      warning: "#f0c674",
      success: "#7fe08d",
      codeBg: "#111612",
    },
  },
  {
    id: "moss",
    name: "Moss",
    nameZh: "苔",
    colorScheme: "dark",
    tokens: {
      ...darkBase,
      bg: "#0c110d",
      bgElevated: "#111810",
      surface: "#172016",
      surfaceHover: "#1f2b1d",
      card: "#1c271a",
      cardFloating: "#273524",
      button: "#324230",
      userMessage: "#3d5038",
      quotation: "#283523",
      border: "#394b35",
      borderStrong: "#536a4b",
      text: "#f1f7ec",
      textSecondary: "#c9d7c0",
      textTertiary: "#91a287",
      brand: "#a9dd7c",
      brandSoft: "rgba(169,221,124,.14)",
      brandText: "#0d1607",
      danger: "#ff8a7a",
      warning: "#e8c56b",
      success: "#a9dd7c",
      codeBg: "#121a10",
    },
  },
  {
    id: "ember",
    name: "Ember",
    nameZh: "余烬",
    colorScheme: "dark",
    tokens: {
      ...darkBase,
      bg: "#100b09",
      bgElevated: "#17100d",
      surface: "#1e1511",
      surfaceHover: "#291c16",
      card: "#241813",
      cardFloating: "#31211a",
      button: "#3d2a20",
      userMessage: "#4c3428",
      quotation: "#33231b",
      border: "#4a3327",
      borderStrong: "#6a4a37",
      text: "#fbf2ec",
      textSecondary: "#dcc7ba",
      textTertiary: "#a98f80",
      brand: "#ffad7a",
      brandSoft: "rgba(255,173,122,.14)",
      brandText: "#1b0d06",
      danger: "#ff7c6e",
      warning: "#f6c177",
      success: "#9bdc8f",
      codeBg: "#160f0c",
    },
  },
  {
    id: "sakura",
    name: "Sakura",
    nameZh: "樱",
    colorScheme: "light",
    tokens: {
      overlay: "rgba(37,22,29,.42)",
      shadow: "0 24px 70px rgba(106,58,76,.18)",
      glow: "0 0 0 1px rgba(80,30,50,.05), 0 18px 50px rgba(106,58,76,.12)",
      bg: "#f7eef0",
      bgElevated: "#fbf5f6",
      surface: "#fff9fa",
      surfaceHover: "#fbe9ed",
      card: "#fffdfd",
      cardFloating: "#ffffff",
      button: "#f3dfe4",
      userMessage: "#f6dbe2",
      quotation: "#f9e8ec",
      border: "#e7cdd4",
      borderStrong: "#d2aab5",
      text: "#2b1c21",
      textSecondary: "#5e4650",
      textTertiary: "#947683",
      brand: "#c94f71",
      brandSoft: "rgba(201,79,113,.12)",
      brandText: "#fffafb",
      danger: "#c73d52",
      warning: "#a7761f",
      success: "#3f8a5b",
      codeBg: "#f8edf0",
    },
  },
  {
    id: "midnight",
    name: "Midnight",
    nameZh: "午夜",
    colorScheme: "dark",
    tokens: {
      ...darkBase,
      bg: "#070a13",
      bgElevated: "#0b1020",
      surface: "#10172a",
      surfaceHover: "#16203a",
      card: "#141d33",
      cardFloating: "#1d2946",
      button: "#263557",
      userMessage: "#33466e",
      quotation: "#1b2742",
      border: "#28395c",
      borderStrong: "#3f5888",
      text: "#f0f4ff",
      textSecondary: "#c1cdea",
      textTertiary: "#8595bb",
      brand: "#8db3ff",
      brandSoft: "rgba(141,179,255,.15)",
      brandText: "#061026",
      danger: "#ff7f9b",
      warning: "#e7c46e",
      success: "#78dfc2",
      codeBg: "#0b1120",
    },
  },
  {
    id: "sand",
    name: "Sand",
    nameZh: "沙",
    colorScheme: "light",
    tokens: {
      overlay: "rgba(44,35,22,.42)",
      shadow: "0 24px 70px rgba(88,64,28,.18)",
      glow: "0 0 0 1px rgba(80,55,20,.05), 0 18px 50px rgba(88,64,28,.12)",
      bg: "#f3ead9",
      bgElevated: "#faf4e8",
      surface: "#fffaf0",
      surfaceHover: "#f4e8d2",
      card: "#fffdf7",
      cardFloating: "#ffffff",
      button: "#ecdfc6",
      userMessage: "#f2e2c4",
      quotation: "#f7ecd8",
      border: "#ddcba9",
      borderStrong: "#c2a87d",
      text: "#2a2118",
      textSecondary: "#5d4d39",
      textTertiary: "#907c61",
      brand: "#a86b1f",
      brandSoft: "rgba(168,107,31,.12)",
      brandText: "#fffaf0",
      danger: "#b34137",
      warning: "#8e6817",
      success: "#4d7a3c",
      codeBg: "#f7efe0",
    },
  },
  {
    id: "violet",
    name: "Violet",
    nameZh: "紫",
    colorScheme: "dark",
    tokens: {
      ...darkBase,
      bg: "#0d0814",
      bgElevated: "#130d1e",
      surface: "#1a1228",
      surfaceHover: "#241a36",
      card: "#201630",
      cardFloating: "#2d2044",
      button: "#3a2a55",
      userMessage: "#4a3670",
      quotation: "#2a1e40",
      border: "#3d2b5b",
      borderStrong: "#5b4384",
      text: "#f5f0ff",
      textSecondary: "#d3c7e9",
      textTertiary: "#9d8cbb",
      brand: "#c0a0ff",
      brandSoft: "rgba(192,160,255,.15)",
      brandText: "#150725",
      danger: "#ff87b7",
      warning: "#e8c878",
      success: "#8fe0b0",
      codeBg: "#110b1b",
    },
  },
  {
    id: "ocean",
    name: "Ocean",
    nameZh: "海",
    colorScheme: "dark",
    tokens: {
      ...darkBase,
      bg: "#061012",
      bgElevated: "#0a171a",
      surface: "#0f2024",
      surfaceHover: "#153037",
      card: "#11252a",
      cardFloating: "#18343b",
      button: "#20454e",
      userMessage: "#2a5964",
      quotation: "#17333a",
      border: "#22505a",
      borderStrong: "#347584",
      text: "#effbfc",
      textSecondary: "#bee0e3",
      textTertiary: "#82aeb4",
      brand: "#65e0d0",
      brandSoft: "rgba(101,224,208,.14)",
      brandText: "#041716",
      danger: "#ff8c86",
      warning: "#e9c66e",
      success: "#65e0d0",
      codeBg: "#081619",
    },
  },
  {
    id: "amber",
    name: "Amber",
    nameZh: "琥珀",
    colorScheme: "dark",
    tokens: {
      ...darkBase,
      bg: "#110d06",
      bgElevated: "#181207",
      surface: "#20180a",
      surfaceHover: "#2c210d",
      card: "#261d0c",
      cardFloating: "#352813",
      button: "#44341a",
      userMessage: "#564322",
      quotation: "#322612",
      border: "#4d3b1e",
      borderStrong: "#70572c",
      text: "#fff8e8",
      textSecondary: "#e3d3b1",
      textTertiary: "#ac9770",
      brand: "#ffd166",
      brandSoft: "rgba(255,209,102,.15)",
      brandText: "#1a1204",
      danger: "#ff8a73",
      warning: "#ffd166",
      success: "#a8d98b",
      codeBg: "#171106",
    },
  },
];

export function getTheme(id: string): Theme {
  return THEMES.find((t) => t.id === id) ?? THEMES[0];
}

export function applyTheme(id: string) {
  const theme = getTheme(id);
  const root = document.documentElement;
  root.dataset.theme = theme.id;
  root.style.colorScheme = theme.colorScheme;
  const t = theme.tokens;
  root.style.setProperty("--bg", t.bg);
  root.style.setProperty("--bg-elevated", t.bgElevated);
  root.style.setProperty("--surface", t.surface);
  root.style.setProperty("--surface-hover", t.surfaceHover);
  root.style.setProperty("--card", t.card);
  root.style.setProperty("--card-floating", t.cardFloating);
  root.style.setProperty("--button", t.button);
  root.style.setProperty("--user-message", t.userMessage);
  root.style.setProperty("--quotation", t.quotation);
  root.style.setProperty("--border", t.border);
  root.style.setProperty("--border-strong", t.borderStrong);
  root.style.setProperty("--text", t.text);
  root.style.setProperty("--text-secondary", t.textSecondary);
  root.style.setProperty("--text-tertiary", t.textTertiary);
  root.style.setProperty("--brand", t.brand);
  root.style.setProperty("--brand-soft", t.brandSoft);
  root.style.setProperty("--brand-text", t.brandText);
  root.style.setProperty("--danger", t.danger);
  root.style.setProperty("--warning", t.warning);
  root.style.setProperty("--success", t.success);
  root.style.setProperty("--shadow", t.shadow);
  root.style.setProperty("--glow", t.glow);
  root.style.setProperty("--overlay", t.overlay);
  root.style.setProperty("--code-bg", t.codeBg);
}
