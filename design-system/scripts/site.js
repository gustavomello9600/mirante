/* ============================================================
   Mirante Design System — comportamento do site
   Sem dependências. O app em Rust/WASM consome os mesmos tokens.
   ============================================================ */

(function () {
  "use strict";

  const root = document.documentElement;
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  /* ---------- Tema ---------- */

  const themeButtons = Array.from(document.querySelectorAll("[data-theme-toggle]"));

  function applyTheme(theme) {
    root.dataset.theme = theme;
    try {
      localStorage.setItem("mirante-theme", theme);
    } catch (e) {}
    themeButtons.forEach(function (button) {
      button.setAttribute("aria-pressed", String(theme === "dark"));
      const label = button.querySelector("[data-theme-label]");
      if (label) label.textContent = theme === "dark" ? "Tema claro" : "Tema escuro";
    });
    updateSwatchValues();
  }

  function updateSwatchValues() {
    const computed = window.getComputedStyle(root);
    document.querySelectorAll(".swatch [data-token]").forEach(function (tokenNode) {
      const token = tokenNode.getAttribute("data-token");
      const button = tokenNode.closest(".swatch");
      if (!token || !button) return;
      const value = computed.getPropertyValue(token).trim();
      if (!value) return;
      const upper = value.toUpperCase();
      button.setAttribute("data-copy", upper);
      const code = button.querySelector("code");
      if (code) code.textContent = upper;
      if (!button.querySelector(".swatch-action")) {
        const action = document.createElement("span");
        action.className = "visually-hidden swatch-action";
        action.textContent = "Copiar cor. ";
        button.insertBefore(action, button.firstChild);
      }
      button.removeAttribute("aria-label");
    });
  }

  themeButtons.forEach(function (button) {
    button.addEventListener("click", function () {
      applyTheme(root.dataset.theme === "dark" ? "light" : "dark");
    });
  });

  applyTheme(root.dataset.theme || "light");

  /* ---------- Menu mobile ---------- */

  const rail = document.getElementById("rail");
  const scrim = document.querySelector("[data-scrim]");
  const menuButtons = Array.from(document.querySelectorAll("[data-menu-toggle]"));

  function setMenu(open) {
    if (!rail) return;
    rail.dataset.open = String(open);
    if (scrim) scrim.hidden = !open;
    menuButtons.forEach(function (button) {
      button.setAttribute("aria-expanded", String(open));
    });
    document.body.style.overflow = open ? "hidden" : "";
  }

  menuButtons.forEach(function (button) {
    button.addEventListener("click", function () {
      setMenu(rail.dataset.open !== "true");
    });
  });

  if (scrim) {
    scrim.addEventListener("click", function () {
      setMenu(false);
    });
  }

  document.addEventListener("keydown", function (event) {
    if (event.key === "Escape") setMenu(false);
  });

  if (rail) {
    rail.querySelectorAll("a").forEach(function (link) {
      link.addEventListener("click", function () {
        if (window.matchMedia("(max-width: 1080px)").matches) setMenu(false);
      });
    });
  }

  window.matchMedia("(min-width: 1081px)").addEventListener("change", function (event) {
    if (event.matches) setMenu(false);
  });

  /* ---------- Scrollspy ---------- */

  const sections = Array.from(document.querySelectorAll("main section[id]"));
  const navLinks = Array.from(document.querySelectorAll("[data-nav]"));
  let ticking = false;

  function updateNav() {
    const marker = window.scrollY + 150;
    let current = null;

    sections.forEach(function (section) {
      const top = section.getBoundingClientRect().top + window.scrollY;
      if (top <= marker) current = section.id;
    });

    navLinks.forEach(function (link) {
      if (current && link.getAttribute("href") === "#" + current) {
        link.setAttribute("aria-current", "true");
      } else {
        link.removeAttribute("aria-current");
      }
    });

    ticking = false;
  }

  window.addEventListener(
    "scroll",
    function () {
      if (!ticking) {
        window.requestAnimationFrame(updateNav);
        ticking = true;
      }
    },
    { passive: true }
  );

  updateNav();

  /* ---------- Toast + copiar ---------- */

  const toast = document.querySelector("[data-toast]");
  let toastTimer = null;

  function showToast(message) {
    if (!toast) return;
    toast.textContent = message;
    toast.dataset.show = "true";
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(function () {
      toast.dataset.show = "false";
    }, 2200);
  }

  function fallbackCopy(text) {
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.top = "-1000px";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    try {
      document.execCommand("copy");
      showToast("Copiado");
    } catch (error) {
      showToast("Não foi possível copiar");
    }
    area.remove();
  }

  function copyText(text) {
    if (navigator.clipboard && window.isSecureContext) {
      navigator.clipboard
        .writeText(text)
        .then(function () {
          showToast("Copiado");
        })
        .catch(function () {
          fallbackCopy(text);
        });
    } else {
      fallbackCopy(text);
    }
  }

  document.querySelectorAll("[data-copy]").forEach(function (button) {
    button.addEventListener("click", function () {
      const value = button.getAttribute("data-copy");
      if (!value) return;
      copyText(value);
      showToast(value.toUpperCase() + " copiado");
    });
  });

  document.querySelectorAll("[data-copy-target]").forEach(function (button) {
    button.addEventListener("click", function () {
      const target = document.querySelector(button.getAttribute("data-copy-target"));
      if (target) copyText(target.textContent.trim());
    });
  });

  document.querySelectorAll("[data-save-story]").forEach(function (button) {
    button.addEventListener("click", function () {
      const saved = button.getAttribute("aria-pressed") !== "true";
      button.setAttribute("aria-pressed", String(saved));
      const label = button.querySelector("span");
      if (label) label.textContent = saved ? "Salvo" : "Salvar";
      showToast(saved ? "História salva" : "Removida dos salvos");
    });
  });

  document.querySelectorAll("[data-share-story]").forEach(function (button) {
    button.addEventListener("click", function () {
      copyText(window.location.href);
    });
  });

  /* ---------- Escala tipográfica ao vivo ---------- */

  const typeTokens = {
    "display-xl": "--text-display-xl",
    "display-l": "--text-display-l",
    "display-m": "--text-display-m",
    headline: "--text-headline",
    title: "--text-title",
    lede: "--text-lede",
    body: "--text-body",
    ui: "--text-ui",
    small: "--text-small",
    caption: "--text-caption"
  };

  function updateTypeSizes() {
    Object.keys(typeTokens).forEach(function (key) {
      const probe = document.createElement("span");
      probe.textContent = "Aa";
      probe.style.position = "absolute";
      probe.style.visibility = "hidden";
      probe.style.pointerEvents = "none";
      probe.style.fontSize = "var(" + typeTokens[key] + ")";
      probe.style.lineHeight = "1";
      document.body.appendChild(probe);
      const size = parseFloat(window.getComputedStyle(probe).fontSize);
      probe.remove();

      document.querySelectorAll('[data-type-size="' + key + '"]').forEach(function (target) {
        target.textContent = Math.round(size) + " px";
      });
    });
  }

  updateTypeSizes();

  let resizeTimer = null;
  window.addEventListener("resize", function () {
    window.clearTimeout(resizeTimer);
    resizeTimer = window.setTimeout(updateTypeSizes, 180);
  });

  /* ---------- Barras de viés e factualidade ---------- */

  const biasLabels = ["Esquerda", "Centro", "Direita", "Sem avaliação"];
  const biasColors = [
    "var(--bias-left)",
    "var(--bias-center)",
    "var(--bias-right)",
    "var(--bias-unknown)"
  ];

  const factLabels = ["Alta", "Mista", "Baixa", "Sem avaliação"];
  const factColors = [
    "var(--fact-high)",
    "var(--fact-mixed)",
    "var(--fact-low)",
    "var(--bias-unknown)"
  ];

  function readValues(chart) {
    const raw = chart.getAttribute("data-values") || "0,0,0,0";
    return raw.split(",").map(function (value) {
      return Number(value.trim()) || 0;
    });
  }

  function renderBiasChart(chart) {
    const values = readValues(chart);
    const bar = chart.querySelector("[data-bias-bar]");
    const legend = chart.querySelector("[data-bias-legend]");
    if (!bar || !legend) return;

    bar.innerHTML = "";
    legend.innerHTML = "";

    const summary = values
      .map(function (value, index) {
        return value + "% " + biasLabels[index].toLowerCase();
      })
      .join(", ");

    bar.setAttribute("aria-label", "Distribuição por viés: " + summary + ".");

    values.forEach(function (value, index) {
      const segment = document.createElement("span");
      segment.style.width = value + "%";
      segment.style.background = biasColors[index];
      segment.setAttribute("aria-hidden", "true");
      segment.title = biasLabels[index] + ": " + value + "%";
      bar.appendChild(segment);

      const item = document.createElement("li");
      const dot = document.createElement("span");
      dot.className = "legend-dot" + (index === 3 ? " legend-dot-hollow" : "");
      dot.style.background = index === 3 ? "transparent" : biasColors[index];
      if (index === 3) dot.style.boxShadow = "inset 0 0 0 1px var(--bias-unknown)";
      const label = document.createElement("span");
      label.textContent = biasLabels[index];
      const amount = document.createElement("span");
      amount.className = "legend-value";
      amount.textContent = value + "%";
      item.append(dot, label, amount);
      legend.appendChild(item);
    });

    chart.setAttribute("data-rendered", "true");
  }

  function renderFactChart(chart) {
    const values = readValues(chart);
    const bar = chart.querySelector("[data-fact-bar]");
    const legend = chart.querySelector("[data-fact-legend]");
    if (!bar || !legend) return;

    bar.innerHTML = "";
    legend.innerHTML = "";

    const summary = values
      .map(function (value, index) {
        return value + "% " + factLabels[index].toLowerCase();
      })
      .join(", ");

    bar.setAttribute("aria-label", "Factualidade: " + summary + ".");

    values.forEach(function (value, index) {
      const segment = document.createElement("span");
      segment.style.width = value + "%";
      segment.style.background = factColors[index];
      segment.setAttribute("aria-hidden", "true");
      segment.title = factLabels[index] + ": " + value + "%";
      bar.appendChild(segment);

      const item = document.createElement("li");
      const square = document.createElement("span");
      square.className = "legend-dot";
      square.style.background = index === 3 ? "transparent" : factColors[index];
      square.style.borderRadius = "1px";
      if (index === 3) square.style.boxShadow = "inset 0 0 0 1px var(--bias-unknown)";
      const label = document.createElement("span");
      label.textContent = factLabels[index];
      const amount = document.createElement("span");
      amount.className = "legend-value";
      amount.textContent = value + "%";
      item.append(square, label, amount);
      legend.appendChild(item);
    });

    chart.setAttribute("data-rendered", "true");
  }

  document.querySelectorAll("[data-bias-chart]").forEach(renderBiasChart);
  document.querySelectorAll("[data-fact-chart]").forEach(renderFactChart);

  /* ---------- Dispersão ---------- */

  const SVG_NS = "http://www.w3.org/2000/svg";

  const scatterData = [
    { name: "Agência Farol", bias: "Esquerda", fact: "Alta", count: 24, x: -1.72, y: 5.0 },
    { name: "Revista Ponte", bias: "Esquerda", fact: "Alta", count: 9, x: -1.28, y: 4.85 },
    { name: "Fio", bias: "Esquerda", fact: "Mista", count: 4, x: -1.92, y: 3.15 },
    { name: "Jornal do Meio", bias: "Centro", fact: "Alta", count: 31, x: -0.06, y: 5.05 },
    { name: "Correio do Vale", bias: "Centro", fact: "Alta", count: 11, x: 0.32, y: 4.7 },
    { name: "Gazeta Federal", bias: "Centro", fact: "Mista", count: 14, x: -0.38, y: 3.0 },
    { name: "Agência Prisma", bias: "Centro", fact: "Alta", count: 5, x: 0.16, y: 4.35 },
    { name: "Rede Sul", bias: "Direita", fact: "Mista", count: 18, x: 1.55, y: 3.35 },
    { name: "Rádio Horizonte", bias: "Direita", fact: "Alta", count: 6, x: 1.18, y: 4.55 },
    { name: "Sul Aberto", bias: "Direita", fact: "Mista", count: 3, x: 1.88, y: 2.75 },
    { name: "Diário da Serra", bias: "Direita", fact: "Baixa", count: 2, x: 1.36, y: 1.45 }
  ];

  const scatterColor = {
    Esquerda: "var(--bias-left)",
    Centro: "var(--bias-center)",
    Direita: "var(--bias-right)",
    "Sem avaliação": "var(--bias-unknown)"
  };

  function svgNode(tag, attrs, parent) {
    const node = document.createElementNS(SVG_NS, tag);
    Object.keys(attrs).forEach(function (key) {
      node.setAttribute(key, attrs[key]);
    });
    if (parent) parent.appendChild(node);
    return node;
  }

  function renderScatter(figure) {
    const svg = figure.querySelector("[data-scatter-svg]");
    const tip = figure.querySelector("[data-chart-tip]");
    if (!svg) return;

    const compact = Boolean(figure.closest(".story-rail"));
    const width = compact ? 360 : 720;
    const height = compact ? 200 : 260;
    const padding = compact
      ? { top: 10, right: 10, bottom: 26, left: 34 }
      : { top: 14, right: 16, bottom: 34, left: 52 };
    const fontAxis = compact ? 10.5 : 10;
    const radiusBase = compact ? 2.6 : 4.0;
    const radiusScale = compact ? 0.6 : 0.82;
    const plotWidth = width - padding.left - padding.right;
    const plotHeight = height - padding.top - padding.bottom;
    const xMin = -2.2;
    const xMax = 2.2;
    const yMax = 5.5;

    svg.setAttribute("viewBox", "0 0 " + width + " " + height);
    svg.innerHTML = "";

    function px(x) {
      return padding.left + ((x - xMin) / (xMax - xMin)) * plotWidth;
    }

    function py(y) {
      return padding.top + (1 - y / yMax) * plotHeight;
    }

    const grid = svgNode("g", { "aria-hidden": "true" }, svg);

    [1.45, 3.2, 5.0].forEach(function (value) {
      svgNode(
        "line",
        {
          x1: padding.left,
          x2: width - padding.right,
          y1: py(value),
          y2: py(value),
          stroke: "var(--rule)",
          "stroke-width": "1"
        },
        grid
      );
      svgNode(
        "text",
        {
          x: padding.left - 7,
          y: py(value) + 3,
          "text-anchor": "end",
          fill: "var(--muted)",
          "font-family": "var(--font-ui)",
          "font-size": String(fontAxis)
        },
        grid
      ).textContent = value === 1.45 ? "Baixa" : value === 3.2 ? "Mista" : "Alta";
    });

    svgNode(
      "line",
      {
        x1: padding.left,
        x2: width - padding.right,
        y1: height - padding.bottom,
        y2: height - padding.bottom,
        stroke: "var(--rule-strong)",
        "stroke-width": "1"
      },
      grid
    );

    svgNode(
      "line",
      {
        x1: px(0),
        x2: px(0),
        y1: padding.top,
        y2: height - padding.bottom,
        stroke: "var(--rule)",
        "stroke-width": "1"
      },
      grid
    );

    ["Esquerda", "Centro", "Direita"].forEach(function (label, index) {
      const x = px(-1.55 + index * 1.55);
      svgNode(
        "text",
        {
          x: x,
          y: height - 10,
          "text-anchor": "middle",
          fill: "var(--muted)",
          "font-family": "var(--font-ui)",
          "font-size": String(fontAxis)
        },
        grid
      ).textContent = label;
    });

    const points = svgNode("g", {}, svg);

    scatterData.forEach(function (point) {
      const cx = px(point.x);
      const cy = py(point.y);
      const radius = radiusBase + Math.sqrt(point.count) * radiusScale;
      const unknown = point.bias === "Sem avaliação";

      const circle = svgNode(
        "circle",
        {
          cx: cx,
          cy: cy,
          r: radius,
          fill: unknown ? "none" : scatterColor[point.bias],
          stroke: unknown ? "var(--bias-unknown)" : "var(--paper-raised)",
          "stroke-width": unknown ? "1.3" : "1.5",
          tabindex: "0",
          role: "img",
          "aria-label":
            point.name +
            ", " +
            point.bias.toLowerCase() +
            ", factualidade " +
            point.fact.toLowerCase() +
            ", " +
            point.count +
            " artigos"
        },
        points
      );

      const title = document.createElementNS(SVG_NS, "title");
      title.textContent =
        point.name + " · " + point.bias + " · " + point.fact + " · " + point.count + " artigos";
      circle.appendChild(title);

      function showTip() {
        if (!tip) return;
        tip.textContent =
          point.name + " · " + point.fact + " · " + point.count + " artigos";
        tip.style.left = (cx / width) * 100 + "%";
        tip.style.top = (cy / height) * 100 + "%";
        tip.dataset.visible = "true";
      }

      function hideTip() {
        if (tip) tip.dataset.visible = "false";
      }

      circle.addEventListener("pointerenter", showTip);
      circle.addEventListener("pointerleave", hideTip);
      circle.addEventListener("focus", showTip);
      circle.addEventListener("blur", hideTip);
    });
  }

  document.querySelectorAll("[data-scatter]").forEach(renderScatter);

  /* ---------- Timeline ---------- */

  const timelineSeries = [
    {
      name: "Esquerda",
      color: "var(--bias-left)",
      values: [4, 5, 6, 5, 8, 7, 9, 18, 22, 19, 14, 11, 9, 8]
    },
    {
      name: "Centro",
      color: "var(--bias-center)",
      values: [3, 4, 5, 6, 6, 7, 8, 14, 17, 16, 13, 10, 9, 8]
    },
    {
      name: "Direita",
      color: "var(--bias-right)",
      values: [2, 3, 3, 4, 5, 4, 6, 7, 9, 12, 18, 15, 11, 9]
    }
  ];

  function renderTimeline(figure) {
    const svg = figure.querySelector("[data-timeline-svg]");
    if (!svg) return;

    const compact = Boolean(figure.closest(".story-rail"));
    const width = compact ? 360 : 720;
    const height = compact ? 180 : 240;
    const padding = compact
      ? { top: 10, right: 10, bottom: 24, left: 30 }
      : { top: 14, right: 16, bottom: 32, left: 40 };
    const fontAxis = compact ? 10.5 : 10;
    const plotWidth = width - padding.left - padding.right;
    const plotHeight = height - padding.top - padding.bottom;
    const maxValue = 24;

    svg.setAttribute("viewBox", "0 0 " + width + " " + height);
    svg.innerHTML = "";

    const grid = svgNode("g", { "aria-hidden": "true" }, svg);

    [0, 10, 20].forEach(function (value) {
      const y = padding.top + (1 - value / maxValue) * plotHeight;
      svgNode(
        "line",
        {
          x1: padding.left,
          x2: width - padding.right,
          y1: y,
          y2: y,
          stroke: "var(--rule)",
          "stroke-width": "1"
        },
        grid
      );
      svgNode(
        "text",
        {
          x: padding.left - 7,
          y: y + 3,
          "text-anchor": "end",
          fill: "var(--muted)",
          "font-family": "var(--font-ui)",
          "font-size": String(fontAxis)
        },
        grid
      ).textContent = String(value);
    });

    const labels = [
      { index: 0, text: "08 out" },
      { index: 3, text: "11 out" },
      { index: 7, text: "15 out" },
      { index: 10, text: "18 out" },
      { index: 13, text: "21 out" }
    ];

    labels.forEach(function (label) {
      const x = padding.left + (label.index / 13) * plotWidth;
      svgNode(
        "text",
        {
          x: x,
          y: height - 8,
          "text-anchor": label.index === 0 ? "start" : label.index === 13 ? "end" : "middle",
          fill: "var(--muted)",
          "font-family": "var(--font-ui)",
          "font-size": String(fontAxis)
        },
        grid
      ).textContent = label.text;
    });

    timelineSeries.forEach(function (series) {
      const points = series.values.map(function (value, index) {
        return {
          x: padding.left + (index / (series.values.length - 1)) * plotWidth,
          y: padding.top + (1 - value / maxValue) * plotHeight
        };
      });

      const d = points
        .map(function (point, index) {
          return (index === 0 ? "M" : "L") + point.x.toFixed(1) + " " + point.y.toFixed(1);
        })
        .join(" ");

      const path = svgNode(
        "path",
        {
          d: d,
          fill: "none",
          stroke: series.color,
          "stroke-width": "1.5",
          "stroke-linecap": "round",
          "stroke-linejoin": "round",
          "vector-effect": "non-scaling-stroke"
        },
        svg
      );

      const title = document.createElementNS(SVG_NS, "title");
      title.textContent = series.name + ": " + series.values.join(", ");
      path.appendChild(title);

      const last = points[points.length - 1];
      svgNode(
        "circle",
        {
          cx: last.x,
          cy: last.y,
          r: 2.2,
          fill: series.color,
          stroke: "var(--paper-raised)",
          "stroke-width": "1"
        },
        svg
      );
    });
  }

  document.querySelectorAll("[data-timeline]").forEach(renderTimeline);

  /* ---------- Respeito a movimento reduzido ---------- */

  if (reduceMotion) {
    document.querySelectorAll(".wordmark, .hero-frame").forEach(function (node) {
      node.style.animation = "none";
    });
  }
})();
