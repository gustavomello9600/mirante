use std::rc::Rc;

use gloo_timers::callback::Timeout;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::api;
use crate::model::{
    Analysis, Article, BiasCounts, Callout, FactCounts, JobView, Lean, ReportFeature,
    ReportParagraph, ReportSection, ScatterPoint, SourceCount, SourceInfo, TimelinePoint, Topic,
};
use crate::web;

#[derive(Clone, PartialEq)]
enum View {
    Home,
    Loading {
        query: String,
        job: Option<JobView>,
    },
    Articles {
        analysis: Analysis,
        selected: Vec<Lean>,
    },
    Empty {
        query: String,
    },
    Report(Analysis),
    Error {
        query: String,
        message: String,
    },
    Saved,
    SourcesLoading,
    Sources(Vec<SourceInfo>),
    SourcesError(String),
}

fn icon(name: &str) -> Html {
    let body = match name {
        "search" => {
            html! { <><circle cx="10.8" cy="10.8" r="6.4"/><path d="m15.6 15.6 4.6 4.6"/></> }
        }
        "arrow-right" => html! { <path d="M4 12h16M14 6l6 6-6 6"/> },
        "arrow-up-right" => html! { <path d="M7 17 17 7M8 7h9v9"/> },
        "chevron-right" => html! { <path d="m9 6 6 6-6 6"/> },
        "menu" => html! { <path d="M4 7h16M4 12h16M4 17h16"/> },
        "close" => html! { <path d="m6 6 12 12M18 6 6 18"/> },
        "sun" => {
            html! { <><circle cx="12" cy="12" r="4"/><path d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M18.7 5.3l-1.4 1.4M6.7 17.3l-1.4 1.4"/></> }
        }
        "moon" => html! { <path d="M20 14.2A8.2 8.2 0 0 1 9.8 4a8.2 8.2 0 1 0 10.2 10.2Z"/> },
        "bookmark" => {
            html! { <path d="M6 4.5A1.5 1.5 0 0 1 7.5 3h9A1.5 1.5 0 0 1 18 4.5V21l-6-4.2L6 21V4.5Z"/> }
        }
        "share" => {
            html! { <><circle cx="6" cy="12" r="2.4"/><circle cx="18" cy="6" r="2.4"/><circle cx="18" cy="18" r="2.4"/><path d="m8.2 10.9 7.6-3.8M8.2 13.1l7.6 3.8"/></> }
        }
        "external" => {
            html! { <><path d="M14 4h6v6M20 4l-8.5 8.5"/><path d="M18 14v4.5A1.5 1.5 0 0 1 16.5 20h-11A1.5 1.5 0 0 1 4 18.5v-11A1.5 1.5 0 0 1 5.5 6H10"/></> }
        }
        "alert" => html! { <><path d="M12 3.5 21 20H3L12 3.5Z"/><path d="M12 9v5M12 17v.2"/></> },
        "check" => html! { <path d="m5 12 4.2 4.2L19 6.5"/> },
        "trash" => html! { <path d="M4 7h16M9 7V4h6v3M7 7l1 13h8l1-13"/> },
        _ => html! { <circle cx="12" cy="12" r="8"/> },
    };
    html! {
        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">{ body }</svg>
    }
}

fn logo_mark() -> Html {
    html! {
        <svg class="brand-mark" viewBox="0 0 32 32" aria-hidden="true">
            <circle cx="16" cy="13" r="6.5"/>
            <path d="M3 24h26"/>
        </svg>
    }
}

fn sentence_case(value: &str) -> String {
    let trimmed = value.trim();
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn plural(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

fn focus_selector(selector: &'static str) {
    Timeout::new(80, move || {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Some(element) = document.query_selector(selector).ok().flatten() else {
            return;
        };
        let _ = element.set_attribute("tabindex", "-1");
        if let Ok(element) = element.dyn_into::<web_sys::HtmlElement>() {
            let _ = element.focus();
        }
    })
    .forget();
}

fn tag(lean: Lean) -> Html {
    html! {
        <span class={classes!("tag", lean.tag_class())}>
            <span class="tag-dot"></span>{ lean.label() }
        </span>
    }
}

fn fact_tag(factuality: crate::model::Factuality) -> Html {
    html! {
        <span class={classes!("tag", factuality.tag_class())}>
            <span class="tag-square"></span>{ factuality.label() }
        </span>
    }
}

// ---------------------------------------------------------------- Header

#[derive(Properties, PartialEq)]
struct HeaderProps {
    pub on_search: Callback<String>,
    pub on_home: Callback<()>,
    pub on_saved: Callback<()>,
    pub on_sources: Callback<()>,
    pub on_theme: Callback<()>,
    pub theme: String,
    pub current_query: String,
    pub active: String,
}

#[function_component(Header)]
fn header(props: &HeaderProps) -> Html {
    let input = use_state(|| props.current_query.clone());
    let oninput = {
        let input = input.clone();
        Callback::from(move |event: InputEvent| {
            let target = event.target_unchecked_into::<HtmlInputElement>();
            input.set(target.value());
        })
    };
    let onsubmit = {
        let input = input.clone();
        let on_search = props.on_search.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let query = input.trim().to_string();
            if !query.is_empty() {
                on_search.emit(query);
            }
        })
    };
    let home = {
        let on_home = props.on_home.clone();
        Callback::from(move |_| on_home.emit(()))
    };
    let saved = {
        let on_saved = props.on_saved.clone();
        Callback::from(move |_| on_saved.emit(()))
    };
    let sources = {
        let on_sources = props.on_sources.clone();
        Callback::from(move |_| on_sources.emit(()))
    };
    let theme = {
        let on_theme = props.on_theme.clone();
        Callback::from(move |_| on_theme.emit(()))
    };

    {
        let input = input.clone();
        let current = props.current_query.clone();
        use_effect_with(current, move |value| {
            if !value.is_empty() {
                input.set(value.clone());
            }
            || ()
        });
    }

    let home_class = if props.active == "home" {
        "is-active"
    } else {
        ""
    };
    let saved_class = if props.active == "saved" {
        "is-active"
    } else {
        ""
    };
    let sources_class = if props.active == "sources" {
        "is-active"
    } else {
        ""
    };

    html! {
        <header class="app-header">
            <div class="app-header-inner">
                <button type="button" class="app-brand" onclick={home.clone()} aria-label="Mirante — início">
                    { logo_mark() }
                    <span>{ "mirante" }</span>
                </button>
                <nav class="app-nav" aria-label="Navegação principal">
                    <button type="button" class={home_class} onclick={home}>{ "Hoje" }</button>
                    <button type="button" class={saved_class} onclick={saved}>{ "Salvos" }</button>
                    <button type="button" class={sources_class} onclick={sources}>{ "Fontes" }</button>
                </nav>
                <div class="app-tools">
                    <form class="header-search" onsubmit={onsubmit}>
                        { icon("search") }
                        <input
                            type="search"
                            aria-label="Buscar um tema"
                            placeholder="Buscar tema"
                            value={(*input).clone()}
                            oninput={oninput}
                        />
                    </form>
                    <button type="button" class="icon-btn theme-button" onclick={theme} aria-label="Alternar tema">
                        { if props.theme == "dark" { icon("sun") } else { icon("moon") } }
                    </button>
                </div>
            </div>
        </header>
    }
}

// ---------------------------------------------------------------- Home

#[derive(Properties, PartialEq)]
struct HomeProps {
    pub topics: Vec<Topic>,
    pub on_search: Callback<String>,
    pub on_sources: Callback<()>,
}

#[function_component(Home)]
fn home(props: &HomeProps) -> Html {
    let input = use_state(String::new);
    let oninput = {
        let input = input.clone();
        Callback::from(move |event: InputEvent| {
            let target = event.target_unchecked_into::<HtmlInputElement>();
            input.set(target.value());
        })
    };
    let onsubmit = {
        let input = input.clone();
        let on_search = props.on_search.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let query = input.trim().to_string();
            if !query.is_empty() {
                on_search.emit(query);
            }
        })
    };
    let sources = {
        let on_sources = props.on_sources.clone();
        Callback::from(move |_| on_sources.emit(()))
    };

    html! {
        <main class="home-page">
            <section class="home-hero">
                <div class="home-hero-main">
                <p class="hero-kicker"><span class="status-dot"></span>{ "Leitura de cobertura · código aberto" }</p>
                <h1 class="home-wordmark">{ "mirante" }</h1>
                <p class="home-tagline">{ "A notícia por inteiro." }</p>
                <p class="home-lede">
                    { "Escolha um tema e veja como veículos de posições diferentes contaram a mesma história. Um relance com fontes, gráficos e o que ficou de fora." }
                </p>
                <form class="home-search" onsubmit={onsubmit}>
                    { icon("search") }
                    <input
                        type="search"
                        aria-label="Buscar um tema"
                        placeholder="Digite um tema. Ex.: eleições 2026"
                        value={(*input).clone()}
                        oninput={oninput}
                    />
                    <button class="btn btn-primary" type="submit">
                        { "Ler a cobertura" }{ icon("arrow-right") }
                    </button>
                </form>
                <div class="home-suggestions">
                    <p class="micro-label">{ "Sugestões para começar" }</p>
                    <div class="topic-grid">
                        { for props.topics.iter().map(|topic| {
                            let query = topic.query.clone();
                            let on_search = props.on_search.clone();
                            let onclick = Callback::from(move |_| on_search.emit(query.clone()));
                            html! {
                                <button type="button" class="topic-card" onclick={onclick}>
                                    <span class="topic-label">{ &topic.label }</span>
                                    <span class="topic-description">{ &topic.description }</span>
                                    { icon("arrow-up-right") }
                                </button>
                            }
                        }) }
                    </div>
                </div>
                </div>
                <aside class="home-aside">
                    <p class="micro-label">{ "Em uma frase" }</p>
                    <blockquote>{ "“Veja o ângulo antes da manchete.”" }</blockquote>
                    <p class="home-aside-note">{ "O Mirante não sentencia a notícia. Ele mostra a amostra, a fonte de cada avaliação e o que é desconhecido." }</p>
                    <dl class="home-facts">
                        <div><dt>{ "Janela" }</dt><dd>{ "7 dias" }</dd></div>
                        <div><dt>{ "Leitura" }</dt><dd>{ "manchetes + metadados" }</dd></div>
                        <div><dt>{ "Avaliações" }</dt><dd>{ "curadoria demo" }</dd></div>
                    </dl>
                </aside>
            </section>

            <section class="home-section">
                <div class="home-section-head">
                    <p class="micro-label">{ "O que você recebe" }</p>
                    <h2>{ "Cobertura, não veredito." }</h2>
                    <p>{ "O Mirante não decide qual versão é a verdadeira. Ele mostra a amostra, o enquadramento de cada grupo e os limites do que é possível concluir." }</p>
                </div>
                <div class="feature-grid">
                    <article class="feature">
                        <span class="feature-index">{ "01" }</span>
                        <h3>{ "Um relance com citações" }</h3>
                        <p>{ "Um texto curto sobre o que foi publicado, com cada afirmação ligada a uma manchete real." }</p>
                    </article>
                    <article class="feature">
                        <span class="feature-index">{ "02" }</span>
                        <h3>{ "O espectro da amostra" }</h3>
                        <p>{ "Quantos artigos vêm de veículos à esquerda, ao centro e à direita — com N e “sem avaliação” visíveis." }</p>
                    </article>
                    <article class="feature">
                        <span class="feature-index">{ "03" }</span>
                        <h3>{ "Factualidade e posição" }</h3>
                        <p>{ "A dispersão mostra cada veículo por viés e factualidade. Centro não é sinônimo de qualidade." }</p>
                    </article>
                    <article class="feature">
                        <span class="feature-index">{ "04" }</span>
                        <h3>{ "O que ficou de fora" }</h3>
                        <p>{ "Assimetrias de cobertura e termos ausentes, sempre com o tamanho da amostra ao lado." }</p>
                    </article>
                </div>
            </section>

            <section class="home-section home-how">
                <div class="home-section-head">
                    <p class="micro-label">{ "Como funciona" }</p>
                    <h2>{ "Três passos, sem feed infinito." }</h2>
                </div>
                <div class="how-grid">
                    <div class="how-step">
                        <span class="how-number">{ "1" }</span>
                        <h3>{ "Escolha um tema" }</h3>
                        <p>{ "Uma pergunta, um assunto ou uma frase. O Mirante busca manchetes recentes em uma amostra de veículos." }</p>
                    </div>
                    <div class="how-step">
                        <span class="how-number">{ "2" }</span>
                        <h3>{ "Veja o conjunto" }</h3>
                        <p>{ "Agrupamos manchetes do mesmo fato, medimos a presença de cada grupo e deixamos o desconhecido visível." }</p>
                    </div>
                    <div class="how-step">
                        <span class="how-number">{ "3" }</span>
                        <h3>{ "Confira as fontes" }</h3>
                        <p>{ "Cada citação abre a matéria original. O relance descreve a cobertura; a leitura final é sua." }</p>
                    </div>
                </div>
            </section>

            <section class="home-methodology">
                <p>
                    <strong>{ "Transparência primeiro." }</strong>
                    { " As avaliações de viés e factualidade desta demonstração são uma curadoria editorial, não uma base licenciada. Em produção, o Mirante foi desenhado para receber AllSides, Ad Fontes, MBFC ou uma metodologia própria auditável." }
                </p>
                <button type="button" class="link-quiet" onclick={sources}>{ "Ver as fontes e a metodologia" }</button>
            </section>
        </main>
    }
}

// ---------------------------------------------------------------- Charts

#[derive(Properties, PartialEq)]
struct BiasBarProps {
    pub bias: BiasCounts,
}

#[function_component(BiasBar)]
fn bias_bar(props: &BiasBarProps) -> Html {
    let total = props.bias.left + props.bias.center + props.bias.right + props.bias.unknown;
    let pct = |value: usize| {
        if total == 0 {
            0.0
        } else {
            value as f64 / total as f64 * 100.0
        }
    };
    html! {
        <div class="bias-chart">
            <div class="bias-bar" role="img" aria-label={format!("Distribuição por viés: {} esquerda, {} centro, {} direita, {} sem avaliação.", props.bias.left, props.bias.center, props.bias.right, props.bias.unknown)}>
                <span style={format!("width:{:.2}%;background:var(--bias-left)", pct(props.bias.left))}></span>
                <span style={format!("width:{:.2}%;background:var(--bias-center)", pct(props.bias.center))}></span>
                <span style={format!("width:{:.2}%;background:var(--bias-right)", pct(props.bias.right))}></span>
                <span style={format!("width:{:.2}%;background:var(--bias-unknown)", pct(props.bias.unknown))}></span>
            </div>
            <ul class="bias-legend">
                <li><span class="legend-dot" style="background:var(--bias-left)"></span>{ "Esquerda" }<span class="legend-value">{ props.bias.left }</span></li>
                <li><span class="legend-dot" style="background:var(--bias-center)"></span>{ "Centro" }<span class="legend-value">{ props.bias.center }</span></li>
                <li><span class="legend-dot" style="background:var(--bias-right)"></span>{ "Direita" }<span class="legend-value">{ props.bias.right }</span></li>
                <li><span class="legend-dot legend-dot-hollow"></span>{ "Sem avaliação" }<span class="legend-value">{ props.bias.unknown }</span></li>
            </ul>
            <p class="chart-note">{ format!("{} artigos na amostra.", total) }</p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct FactBarProps {
    pub factuality: FactCounts,
}

#[function_component(FactBar)]
fn fact_bar(props: &FactBarProps) -> Html {
    let total = props.factuality.high
        + props.factuality.mixed
        + props.factuality.low
        + props.factuality.unknown;
    let pct = |value: usize| {
        if total == 0 {
            0.0
        } else {
            value as f64 / total as f64 * 100.0
        }
    };
    html! {
        <div class="fact-chart">
            <div class="fact-bar" role="img" aria-label={format!("Factualidade: {} alta, {} mista, {} baixa, {} sem avaliação.", props.factuality.high, props.factuality.mixed, props.factuality.low, props.factuality.unknown)}>
                <span style={format!("width:{:.2}%;background:var(--fact-high)", pct(props.factuality.high))}></span>
                <span style={format!("width:{:.2}%;background:var(--fact-mixed)", pct(props.factuality.mixed))}></span>
                <span style={format!("width:{:.2}%;background:var(--fact-low)", pct(props.factuality.low))}></span>
                <span style={format!("width:{:.2}%;background:var(--bias-unknown)", pct(props.factuality.unknown))}></span>
            </div>
            <ul class="fact-legend">
                <li><span class="legend-dot" style="background:var(--fact-high);border-radius:1px"></span>{ "Alta" }<span class="legend-value">{ props.factuality.high }</span></li>
                <li><span class="legend-dot" style="background:var(--fact-mixed);border-radius:1px"></span>{ "Mista" }<span class="legend-value">{ props.factuality.mixed }</span></li>
                <li><span class="legend-dot" style="background:var(--fact-low);border-radius:1px"></span>{ "Baixa" }<span class="legend-value">{ props.factuality.low }</span></li>
                <li><span class="legend-dot legend-dot-hollow"></span>{ "Sem avaliação" }<span class="legend-value">{ props.factuality.unknown }</span></li>
            </ul>
            <p class="chart-note">{ format!("{} artigos na amostra.", total) }</p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ScatterProps {
    pub points: Vec<ScatterPoint>,
}

#[function_component(ScatterChart)]
fn scatter_chart(props: &ScatterProps) -> Html {
    let mut circles = Vec::new();
    for (index, point) in props.points.iter().enumerate() {
        let jitter_x = ((index % 3) as f64 - 1.0) * 7.0;
        let jitter_y = (((index / 3) % 3) as f64 - 1.0) * 5.0;
        let x = match point.lean {
            Lean::Left => 88.0,
            Lean::Center => 180.0,
            Lean::Right => 272.0,
            Lean::Unknown => 180.0,
        } + jitter_x;
        let y = match point.factuality {
            crate::model::Factuality::High => 48.0,
            crate::model::Factuality::Mixed => 112.0,
            crate::model::Factuality::Low => 176.0,
            crate::model::Factuality::Unknown => 176.0,
        } + jitter_y;
        let radius = (4.0 + (point.count as f64).sqrt() * 1.6).min(14.0);
        let fill = match point.lean {
            Lean::Left => "var(--bias-left)",
            Lean::Center => "var(--bias-center)",
            Lean::Right => "var(--bias-right)",
            Lean::Unknown => "transparent",
        };
        circles.push(html! {
            <circle
                class={classes!("scatter-point", (point.lean == Lean::Unknown).then_some("is-unknown"))}
                cx={format!("{x}")}
                cy={format!("{y}")}
                r={format!("{radius}")}
                fill={fill}
            >
                <title>{ format!("{} · {} · {} artigos", point.name, point.factuality.label(), point.count) }</title>
            </circle>
        });
    }

    html! {
        <figure class="chart-figure">
            <figcaption class="chart-head">
                <span class="chart-title">{ "Viés × factualidade" }</span>
                <span class="chart-sub">{ format!("{} veículos", props.points.len()) }</span>
            </figcaption>
            <div class="scatter-wrap">
                <svg class="scatter-svg" viewBox="0 0 360 220" role="img" aria-label="Dispersão de veículos por viés e factualidade. O tamanho do ponto indica volume de artigos.">
                    <line class="scatter-grid" x1="52" y1="48" x2="340" y2="48"/>
                    <line class="scatter-grid" x1="52" y1="112" x2="340" y2="112"/>
                    <line class="scatter-grid" x1="52" y1="176" x2="340" y2="176"/>
                    <line class="scatter-axis" x1="52" y1="196" x2="340" y2="196"/>
                    <text class="chart-axis-label chart-axis-end" x="44" y="51">{ "Alta" }</text>
                    <text class="chart-axis-label chart-axis-end" x="44" y="115">{ "Mista" }</text>
                    <text class="chart-axis-label chart-axis-end" x="44" y="179">{ "Baixa" }</text>
                    <text class="chart-axis-label" x="88" y="212">{ "Esquerda" }</text>
                    <text class="chart-axis-label" x="180" y="212">{ "Centro" }</text>
                    <text class="chart-axis-label" x="272" y="212">{ "Direita" }</text>
                    { for circles }
                </svg>
            </div>
            <div class="chart-legend" aria-hidden="true">
                <span><i class="legend-dot" style="background:var(--bias-left)"></i>{ "Esquerda" }</span>
                <span><i class="legend-dot" style="background:var(--bias-center)"></i>{ "Centro" }</span>
                <span><i class="legend-dot" style="background:var(--bias-right)"></i>{ "Direita" }</span>
            </div>
            <p class="chart-note">{ "Tamanho = volume de artigos. Veículos sem avaliação ficam fora do gráfico." }</p>
        </figure>
    }
}

#[derive(Properties, PartialEq)]
struct TimelineProps {
    pub points: Vec<TimelinePoint>,
}

#[function_component(TimelineChart)]
fn timeline_chart(props: &TimelineProps) -> Html {
    let max = props
        .points
        .iter()
        .flat_map(|point| [point.left, point.center, point.right])
        .max()
        .unwrap_or(1)
        .max(1) as f64;

    let width = 320.0;
    let height = 150.0;
    let left = 34.0;
    let top = 14.0;
    let step = if props.points.len() > 1 {
        width / (props.points.len() - 1) as f64
    } else {
        0.0
    };

    let series = [
        ("left", "var(--bias-left)"),
        ("center", "var(--bias-center)"),
        ("right", "var(--bias-right)"),
    ];
    let mut paths = Vec::new();
    for (key, color) in series {
        let coords: Vec<(f64, f64)> = props
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                let value = match key {
                    "left" => point.left,
                    "center" => point.center,
                    _ => point.right,
                } as f64;
                (
                    left + index as f64 * step,
                    top + (1.0 - value / max) * height,
                )
            })
            .collect();
        let d = coords
            .iter()
            .enumerate()
            .map(|(index, (x, y))| {
                format!("{}{:.1} {:.1}", if index == 0 { "M" } else { "L" }, x, y)
            })
            .collect::<Vec<_>>()
            .join(" ");
        paths.push(html! { <path class="timeline-line" d={d} stroke={color} /> });
    }

    let labels: Vec<Html> = if props.points.is_empty() {
        Vec::new()
    } else {
        let indexes = [0, props.points.len() / 2, props.points.len() - 1];
        indexes
            .iter()
            .map(|index| {
                let point = &props.points[*index];
                html! {
                    <text class="chart-axis-label" x={format!("{:.1}", left + *index as f64 * step)} y="178">
                        { &point.date }
                    </text>
                }
            })
            .collect()
    };

    html! {
        <figure class="chart-figure">
            <figcaption class="chart-head">
                <span class="chart-title">{ "Volume de cobertura" }</span>
                <span class="chart-sub">{ format!("{} dias", props.points.len()) }</span>
            </figcaption>
            <svg class="timeline-svg" viewBox="0 0 360 190" role="img" aria-label="Volume diário de artigos por grupo de viés.">
                <line class="scatter-grid" x1="34" y1="14" x2="354" y2="14"/>
                <line class="scatter-grid" x1="34" y1="89" x2="354" y2="89"/>
                <line class="scatter-grid" x1="34" y1="164" x2="354" y2="164"/>
                { for paths }
                { for labels }
            </svg>
            <div class="chart-legend" aria-hidden="true">
                <span><i class="legend-dot" style="background:var(--bias-left)"></i>{ "Esquerda" }</span>
                <span><i class="legend-dot" style="background:var(--bias-center)"></i>{ "Centro" }</span>
                <span><i class="legend-dot" style="background:var(--bias-right)"></i>{ "Direita" }</span>
            </div>
            <p class="chart-note">{ "Contagem diária de artigos. Picos são atenção editorial, não verdade." }</p>
        </figure>
    }
}

// ---------------------------------------------------------------- Report

#[derive(Properties, PartialEq)]
struct ReportProps {
    pub analysis: Analysis,
    pub saved: bool,
    pub on_save: Callback<()>,
    pub on_share: Callback<()>,
    pub on_retry: Callback<()>,
    pub on_open_articles: Callback<Vec<Lean>>,
}

#[function_component(ReportView)]
fn report_view(props: &ReportProps) -> Html {
    let analysis = &props.analysis;
    let articles = Rc::new(analysis.articles.clone());
    let total = analysis.stats.total_articles;
    let unknown_pct = if total == 0 {
        0.0
    } else {
        analysis.stats.unknown_articles as f64 / total as f64 * 100.0
    };

    let save = {
        let on_save = props.on_save.clone();
        Callback::from(move |_| on_save.emit(()))
    };
    let share = {
        let on_share = props.on_share.clone();
        Callback::from(move |_| on_share.emit(()))
    };

    use_effect_with((), |_| {
        focus_selector(".story-head h1");
        || ()
    });

    html! {
        <>
            <main class="report-page">
                <nav class="crumbs" aria-label="Trilha">
                    <span>{ "Cobertura" }</span>{ icon("chevron-right") }<span>{ sentence_case(&analysis.query) }</span>
                </nav>

                <header class="story-head">
                    <p class="story-kicker">
                        <span>{ plural(analysis.stats.total_articles, "artigo", "artigos") }</span>
                        <span>{ plural(analysis.stats.total_sources, "veículo", "veículos") }</span>
                        <span>{ format!("últimos {} dias", analysis.stats.window_days) }</span>
                        <span>{ if analysis.cached { "do cache" } else { "gerado agora" } }</span>
                        if analysis.stats.deep_read > 0 {
                            <span>{ format!("{} na íntegra", plural(analysis.stats.deep_read, "lida", "lidas")) }</span>
                        }
                    </p>
                    <h1>{ sentence_case(&analysis.query) }</h1>
                    <p class="story-lede">
                        { format!("Uma leitura de {} de {}. O relance descreve a cobertura, não a realidade; os limites estão ao final.", plural(analysis.stats.total_articles, "manchete", "manchetes"), plural(analysis.stats.total_sources, "veículo", "veículos")) }
                    </p>
                    <div class="story-byline">
                        <span>{ format!("Relance {} · {}", if analysis.mode == "llm" { "gerado por modelo" } else { "extrativo" }, web::format_time(&analysis.generated_at)) }</span>
                        <div class="story-actions">
                            <button type="button" class={classes!("btn", "btn-secondary", "btn-sm")} onclick={save} aria-pressed={props.saved.to_string()}>
                                { icon("bookmark") }<span>{ if props.saved { "Salvo" } else { "Salvar" } }</span>
                            </button>
                            <button type="button" class="btn btn-ghost btn-sm" onclick={share}>
                                { icon("share") }{ "Compartilhar" }
                            </button>
                        </div>
                    </div>
                </header>

                <div class="story-layout">
                    <div class="story-main">
                        <article class="report report-plain">
                            <div class="report-head">
                                <h2 class="report-title">{ &analysis.report.title }</h2>
                                <p class="report-meta">
                                    <span>{ &analysis.report.dek }</span>
                                    <span>{ if analysis.mode == "llm" { "modelo + dados" } else { "extrativo + dados" } }</span>
                                </p>
                            </div>
                            { render_report_body(analysis, &articles, props.on_open_articles.clone()) }
                            <div class="report-limitations">
                                <p class="micro-label">{ "Limitações" }</p>
                                <ul>
                                    { for analysis.report.limitations.iter().map(|item| html! { <li>{ item }</li> }) }
                                </ul>
                            </div>
                            { if analysis.mode != "llm" {
                                html! {
                                    <p class="report-footnote">
                                        { "Relance extrativo: os textos são montados a partir das manchetes e dos números calculados. Para análise generativa, conecte um modelo no servidor." }
                                    </p>
                                }
                            } else { Html::default() } }
                        </article>

                        <section class="sources-block" aria-labelledby="fontes-title">
                            <div class="sources-head">
                                <h2 id="fontes-title">{ "Fontes nesta cobertura" }</h2>
                                <span>{ format!("{} veículos · {} artigos", analysis.stats.total_sources, analysis.stats.total_articles) }</span>
                            </div>
                            { render_source_rows(&analysis.stats.top_sources) }
                            if analysis.stats.top_sources.len() < analysis.stats.total_sources {
                                <p class="chart-note">{ format!("Mostrando os {} veículos com mais artigos; a amostra completa tem {}.", analysis.stats.top_sources.len(), analysis.stats.total_sources) }</p>
                            }
                        </section>

                        <details class="articles-block">
                            <summary>
                                <span>{ format!("Ver as {} manchetes da amostra", analysis.articles.len()) }</span>
                                { icon("chevron-right") }
                            </summary>
                            <div class="article-list">
                                { for analysis.articles.iter().take(80).map(render_article_row) }
                            </div>
                        </details>
                    </div>

                    <aside class="story-rail" aria-label="Evidências da cobertura">
                        <section class="rail-panel">
                            <div class="chart-head"><h2 class="chart-title">{ "Espectro" }</h2><span class="chart-sub">{ format!("{} artigos", total) }</span></div>
                            <BiasBar bias={analysis.stats.bias.clone()} />
                            <p class="chart-note">{ format!("{:.0}% sem avaliação de viés.", unknown_pct) }</p>
                        </section>
                        <section class="rail-panel">
                            <div class="chart-head"><h2 class="chart-title">{ "Factualidade" }</h2><span class="chart-sub">{ format!("{} artigos", total) }</span></div>
                            <FactBar factuality={analysis.stats.factuality.clone()} />
                        </section>
                        <section class="rail-panel">
                            <ScatterChart points={analysis.scatter.clone()} />
                        </section>
                        <section class="rail-panel">
                            <TimelineChart points={analysis.timeline.clone()} />
                        </section>
                        <div class="rail-note">
                            <p><strong>{ "Como ler." }</strong>{ " Percentuais usam artigos, não veículos. Se um grupo tem poucos artigos, leia como indício. “Sem avaliação” fica fora das barras coloridas." }</p>
                        </div>
                    </aside>
                </div>

            </main>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct SpectrumProps {
    pub analysis: Analysis,
    pub on_open: Callback<Vec<Lean>>,
}

fn articles_for_lean(analysis: &Analysis, lean: Lean) -> Vec<Article> {
    let mut articles: Vec<Article> = analysis
        .articles
        .iter()
        .filter(|article| article.lean == lean)
        .cloned()
        .collect();
    articles.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Top 3 com diversidade de fontes; só repete veículo se não houver alternativa.
    let mut selected = Vec::new();
    let mut domains = std::collections::HashSet::new();
    for article in &articles {
        if domains.insert(article.domain.clone()) {
            selected.push(article.clone());
        }
        if selected.len() == 3 {
            break;
        }
    }
    if selected.len() < 3 {
        for article in &articles {
            if !selected.iter().any(|item| item.id == article.id) {
                selected.push(article.clone());
            }
            if selected.len() == 3 {
                break;
            }
        }
    }
    selected
}

fn lean_count(bias: &BiasCounts, lean: Lean) -> usize {
    match lean {
        Lean::Left => bias.left,
        Lean::Center => bias.center,
        Lean::Right => bias.right,
        Lean::Unknown => bias.unknown,
    }
}

#[function_component(SpectrumHighlights)]
fn spectrum_highlights(props: &SpectrumProps) -> Html {
    let analysis = &props.analysis;
    let all = {
        let on_open = props.on_open.clone();
        Callback::from(move |_| on_open.emit(Vec::new()))
    };
    let groups: Vec<(Lean, Vec<Article>, usize)> =
        [Lean::Left, Lean::Center, Lean::Right, Lean::Unknown]
            .iter()
            .map(|lean| {
                let lean = *lean;
                (
                    lean,
                    articles_for_lean(analysis, lean),
                    lean_count(&analysis.stats.bias, lean),
                )
            })
            .filter(|(_, articles, count)| *count > 0 && !articles.is_empty())
            .collect();
    let empty_groups: Vec<Lean> = [Lean::Left, Lean::Center, Lean::Right, Lean::Unknown]
        .iter()
        .copied()
        .filter(|lean| !groups.iter().any(|(group, _, _)| group == lean))
        .collect();

    html! {
        <section class="spectrum-highlights" aria-labelledby="spectrum-highlights-title">
            <div class="spectrum-highlights-head">
                <div>
                    <p class="micro-label">{ "Top 3 por espectro" }</p>
                    <h2 id="spectrum-highlights-title">{ "As matérias que mais explicam cada lado." }</h2>
                    <p class="spectrum-highlights-lede">
                        { "Seleção por centralidade no agrupamento, factualidade da fonte, recência e leitura profunda. A posição é da fonte, não do texto." }
                    </p>
                </div>
                <button type="button" class="btn btn-secondary btn-sm" onclick={all}>
                    { "Abrir todas as matérias" }{ icon("arrow-right") }
                </button>
            </div>

            <div class="spectrum-columns">
                { for groups.iter().map(|(lean, articles, count)| {
                    let lean = *lean;
                    let count = *count;
                    let open = {
                        let on_open = props.on_open.clone();
                        Callback::from(move |_| on_open.emit(vec![lean]))
                    };
                    html! {
                        <article class={classes!("spectrum-column", format!("spectrum-column-{}", lean.tag_class()))}>
                            <header class="spectrum-column-head">
                                { tag(lean) }
                                <span class="spectrum-count">{ plural(count, "artigo", "artigos") }</span>
                            </header>
                            <ol class="spectrum-list">
                                { for articles.iter().take(3).enumerate().map(|(index, article)| {
                                    html! {
                                        <li class="spectrum-card">
                                            <span class="spectrum-rank">{ format!("{:02}", index + 1) }</span>
                                            <div class="spectrum-card-body">
                                                <a href={article.url.clone()} target="_blank" rel="noopener noreferrer">{ &article.title }</a>
                                                <p class="spectrum-meta">
                                                    <span>{ &article.source }</span>
                                                    <span>{ web::relative_time(article.published_at.as_deref().unwrap_or("")) }</span>
                                                </p>
                                                if article.deep_read {
                                                    <span class="deep-badge">{ "Leitura profunda" }</span>
                                                }
                                            </div>
                                        </li>
                                    }
                                }) }
                            </ol>
                            <button type="button" class="spectrum-open" onclick={open}>
                                { if count == 1 { "Ver a matéria".to_string() } else { format!("Ver todas as {count} matérias") } }
                                { icon("arrow-right") }
                            </button>
                        </article>
                    }
                }) }
            </div>
            if !empty_groups.is_empty() {
                <p class="spectrum-empty-note">
                    { "Sem amostra nesta cobertura: " }
                    { empty_groups.iter().map(|lean| lean.label()).collect::<Vec<_>>().join(", ") }
                    { ". Não atribuímos posição a fontes oficiais, empresas ou veículos sem base editorial." }
                </p>
            }
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct ArticlesProps {
    pub analysis: Analysis,
    pub selected: Vec<Lean>,
    pub on_back: Callback<()>,
}

#[function_component(AllArticlesView)]
fn all_articles_view(props: &ArticlesProps) -> Html {
    let selected = use_state(|| props.selected.clone());
    let recent_first = use_state(|| false);

    use_effect_with((), |_| {
        let timeout = Timeout::new(80, || {
            if let Some(window) = web_sys::window() {
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
            focus_selector(".articles-head h1");
        });
        move || drop(timeout)
    });

    let toggle = {
        let selected = selected.clone();
        Callback::from(move |lean: Lean| {
            let mut next = (*selected).clone();
            if let Some(position) = next.iter().position(|item| *item == lean) {
                next.remove(position);
            } else {
                next.push(lean);
            }
            selected.set(next);
        })
    };
    let clear = {
        let selected = selected.clone();
        Callback::from(move |_| selected.set(Vec::new()))
    };
    let back = {
        let on_back = props.on_back.clone();
        Callback::from(move |_| on_back.emit(()))
    };
    let sort_recent = {
        let recent_first = recent_first.clone();
        Callback::from(move |_| recent_first.set(true))
    };
    let sort_relevant = {
        let recent_first = recent_first.clone();
        Callback::from(move |_| recent_first.set(false))
    };

    let mut visible: Vec<&Article> = props
        .analysis
        .articles
        .iter()
        .filter(|article| selected.is_empty() || selected.contains(&article.lean))
        .collect();
    if *recent_first {
        visible.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    } else {
        visible.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    let sources: std::collections::HashSet<&str> = visible
        .iter()
        .map(|article| article.domain.as_str())
        .collect();
    let total = props.analysis.articles.len();
    let shown = visible.len();
    let filter_label = if selected.is_empty() {
        "Todos os espectros".to_string()
    } else {
        selected
            .iter()
            .map(|lean| lean.label())
            .collect::<Vec<_>>()
            .join(" + ")
    };

    html! {
        <main class="articles-page">
            <nav class="crumbs" aria-label="Trilha">
                <button type="button" class="crumb-back" onclick={back.clone()}>{ "Cobertura" }</button>
                { icon("chevron-right") }
                <span>{ sentence_case(&props.analysis.query) }</span>
                { icon("chevron-right") }
                <span>{ "Todas as matérias" }</span>
            </nav>

            <header class="articles-head">
                <button type="button" class="back-link" onclick={back.clone()}>
                    { icon("arrow-right") }{ "Voltar ao relance" }
                </button>
                <p class="micro-label">{ "Amostra completa" }</p>
                <h1>{ "Todas as matérias." }</h1>
                <p class="articles-lede">
                    { format!("{}, com a posição de cada fonte ao lado. Filtre um ou mais espectros para comparar o que cada campo escolheu destacar.", format!("{} de {}", plural(total, "artigo", "artigos"), plural(props.analysis.stats.total_sources, "veículo", "veículos"))) }
                </p>
            </header>

            <section class="article-filters" aria-label="Filtrar matérias por espectro">
                <div class="filter-row">
                    <span class="filter-label">{ "Espectro" }</span>
                    <div class="filter-chips">
                        <button
                            type="button"
                            class={classes!("filter-chip", "filter-chip-all", if selected.is_empty() { "is-active" } else { "" })}
                            aria-pressed={selected.is_empty().to_string()}
                            onclick={clear.clone()}
                        >
                            <span class="filter-chip-label">{ "Todas" }</span>
                            <span class="filter-chip-count">{ total }</span>
                        </button>
                        { for [Lean::Left, Lean::Center, Lean::Right, Lean::Unknown].iter().map(|lean| {
                            let lean = *lean;
                            let count = lean_count(&props.analysis.stats.bias, lean);
                            let active = selected.contains(&lean);
                            let onclick = {
                                let toggle = toggle.clone();
                                Callback::from(move |_| toggle.emit(lean))
                            };
                            html! {
                                <button
                                    type="button"
                                    class={classes!("filter-chip", lean.tag_class(), if active { "is-active" } else { "" })}
                                    aria-pressed={active.to_string()}
                                    onclick={onclick}
                                    disabled={count == 0}
                                >
                                    <span class="filter-chip-dot"></span>
                                    <span class="filter-chip-label">{ lean.label() }</span>
                                    <span class="filter-chip-count">{ count }</span>
                                </button>
                            }
                        }) }
                    </div>
                </div>
                <div class="filter-row filter-row-secondary">
                    <p class="filter-summary" aria-live="polite">
                        <strong>{ format!("{shown} de {total}") }</strong>
                        { format!(" matérias · {} · {filter_label}", plural(sources.len(), "veículo", "veículos")) }
                    </p>
                    <div class="sort-toggle" role="group" aria-label="Ordenar matérias">
                        <button type="button" class={classes!(if !*recent_first { "is-active" } else { "" })} aria-pressed={(!*recent_first).to_string()} onclick={sort_relevant}>{ "Relevância" }</button>
                        <button type="button" class={classes!(if *recent_first { "is-active" } else { "" })} aria-pressed={(*recent_first).to_string()} onclick={sort_recent}>{ "Mais recentes" }</button>
                    </div>
                </div>
            </section>

            if visible.is_empty() {
                <div class="empty-state articles-empty">
                    <span class="empty-mark">{ "—" }</span>
                    <h2>{ "Nenhuma matéria neste filtro" }</h2>
                    <p>{ "A amostra não tem artigos com a combinação de espectros escolhida." }</p>
                    <button type="button" class="btn btn-secondary" onclick={clear.clone()}>{ "Ver todas as matérias" }</button>
                </div>
            } else {
                <div class="article-grid">
                    { for visible.iter().map(|article| {
                        let published = article.published_at.as_deref().unwrap_or("");
                        html! {
                            <article class="coverage-card">
                                <div class="coverage-card-top">
                                    <div class="coverage-source">
                                        <strong>{ &article.source }</strong>
                                        <span>{ web::relative_time(published) }</span>
                                    </div>
                                    <div class="coverage-tags">
                                        { tag(article.lean) }
                                        { fact_tag(article.factuality) }
                                    </div>
                                </div>
                                <a class="coverage-title" href={article.url.clone()} target="_blank" rel="noopener noreferrer">{ &article.title }</a>
                                if let Some(excerpt) = &article.excerpt {
                                    <p class="coverage-excerpt">{ excerpt }</p>
                                } else if let Some(snippet) = &article.snippet {
                                    <p class="coverage-snippet">{ snippet }</p>
                                }
                                <div class="coverage-foot">
                                    <div class="coverage-badges">
                                        if article.deep_read {
                                            <span class="deep-badge">{ "Leitura profunda" }</span>
                                        }
                                        if let Some(owner) = &article.ownership {
                                            <span class="ownership-note">{ owner }</span>
                                        }
                                    </div>
                                    <a class="coverage-open" href={article.url.clone()} target="_blank" rel="noopener noreferrer" aria-label={format!("Abrir matéria de {}", article.source)}>
                                        { "Abrir" }{ icon("external") }
                                    </a>
                                </div>
                            </article>
                        }
                    }) }
                </div>
            }
        </main>
    }
}

fn render_report_body(
    analysis: &Analysis,
    articles: &Rc<Vec<Article>>,
    on_open: Callback<Vec<Lean>>,
) -> Html {
    let mut chunks: Vec<Html> = Vec::new();
    let mut feature_rendered = false;
    let mut spectrum_rendered = false;
    let mut first_section = true;

    for section in &analysis.report.sections {
        let heading = section.heading.to_lowercase();
        if heading.contains("lado") {
            continue;
        }
        if !spectrum_rendered && heading.contains("ficou de fora") {
            chunks.push(html! {
                <SpectrumHighlights analysis={analysis.clone()} on_open={on_open.clone()} />
            });
            spectrum_rendered = true;
        }
        chunks.push(render_section(section, articles));
        if !feature_rendered && (heading.contains("aconteceu") || first_section) {
            if let Some(feature) = &analysis.report.feature {
                chunks.push(render_feature(feature, articles));
            }
            feature_rendered = true;
        }
        first_section = false;
    }

    if !feature_rendered {
        if let Some(feature) = &analysis.report.feature {
            chunks.push(render_feature(feature, articles));
        }
    }
    if !spectrum_rendered {
        chunks.push(html! {
            <SpectrumHighlights analysis={analysis.clone()} on_open={on_open} />
        });
    }

    html! { <>{ for chunks }</> }
}

fn render_feature(feature: &ReportFeature, articles: &Rc<Vec<Article>>) -> Html {
    html! {
        <section class="report-feature">
            <p class="report-feature-kicker">{ &feature.kicker }</p>
            <h2>{ &feature.title }</h2>
            if !feature.dek.is_empty() {
                <p class="report-feature-dek">{ &feature.dek }</p>
            }
            <div class="report-feature-body">
                { for feature.body.iter().map(|paragraph| render_paragraph(paragraph, articles)) }
            </div>
        </section>
    }
}

fn render_section(section: &ReportSection, articles: &Rc<Vec<Article>>) -> Html {
    html! {
        <section class="report-section">
            <h3>{ &section.heading }</h3>
            { for section.body.iter().map(|paragraph| render_paragraph(paragraph, articles)) }
            if let Some(callout) = &section.callout {
                { render_callout(callout) }
            }
        </section>
    }
}

fn split_embedded_label(text: &str) -> (Option<String>, String) {
    for prefix in ["Esquerda", "Centro", "Direita", "Sem avaliação"] {
        if text.starts_with(prefix) {
            if let Some((head, tail)) = text.split_once(": ") {
                if head.contains("artigos") && head.len() < 80 {
                    return (Some(head.to_string()), tail.to_string());
                }
            }
        }
    }
    (None, text.to_string())
}

fn render_paragraph(paragraph: &ReportParagraph, articles: &Rc<Vec<Article>>) -> Html {
    let (label, body) = match &paragraph.label {
        Some(label) => (Some(label.clone()), paragraph.text.clone()),
        None => split_embedded_label(&paragraph.text),
    };
    let citations = if paragraph.citations.is_empty() {
        Html::default()
    } else {
        html! {
            <sup class="citations">
                { for paragraph.citations.iter().filter_map(|index| {
                    let article = articles.get(index.saturating_sub(1))?;
                    Some(html! {
                        <a href={article.url.clone()} target="_blank" rel="noopener noreferrer" title={article.title.clone()}>{ format!("[{}]", index) }</a>
                    })
                }) }
            </sup>
        }
    };
    html! {
        <div class="report-paragraph">
            if let Some(label) = &label {
                <p class="report-label">{ label }</p>
            }
            <p>{ &body }{ citations }</p>
        </div>
    }
}

fn render_callout(callout: &Callout) -> Html {
    html! {
        <aside class="callout">
            <p class="callout-label">{ &callout.label }</p>
            <p class="callout-title">{ &callout.title }</p>
            <p class="callout-body">{ &callout.body }</p>
        </aside>
    }
}

fn render_source_rows(sources: &[SourceCount]) -> Html {
    html! {
        <div class="source-table">
            <div class="source-head" aria-hidden="true">
                <span>{ "Veículo" }</span><span>{ "Viés" }</span><span>{ "Factualidade" }</span><span class="source-count">{ "Artigos" }</span><span></span>
            </div>
            { for sources.iter().map(|source| {
                let search = if source.domain.is_empty() { "https://www.google.com".to_string() } else { format!("https://{}", source.domain) };
                html! {
                    <div class="source-row">
                        <div class="source-name">
                            <a href={search.clone()} target="_blank" rel="noopener noreferrer">{ &source.name }</a>
                            <span>{ &source.domain }</span>
                        </div>
                        { tag(source.lean) }
                        { fact_tag(source.factuality) }
                        <span class="source-count">{ source.count }</span>
                        <a class="source-link" href={search} target="_blank" rel="noopener noreferrer" aria-label={format!("Abrir informações sobre {}", source.name)}>{ icon("external") }</a>
                    </div>
                }
            }) }
        </div>
    }
}

fn render_article_row(article: &Article) -> Html {
    html! {
        <div class="article-row">
            <div class="article-row-main">
                <a href={article.url.clone()} target="_blank" rel="noopener noreferrer">{ &article.title }</a>
                <span>{ format!("{} · {}", article.source, web::relative_time(article.published_at.as_deref().unwrap_or(""))) }</span>
                if let Some(excerpt) = &article.excerpt {
                    <p class="article-row-excerpt">{ excerpt }</p>
                }
            </div>
            <div class="article-row-tags">
                { tag(article.lean) }
                { fact_tag(article.factuality) }
                if article.deep_read {
                    <span class="deep-badge">{ "Leitura profunda" }</span>
                }
            </div>
        </div>
    }
}

// ---------------------------------------------------------------- States

const LOADING_STEPS: [(&str, &str); 6] = [
    (
        "Buscar manchetes",
        "Google News, Bing News e veículos avaliados.",
    ),
    (
        "Avaliar as fontes",
        "Viés, factualidade, recência e diversidade.",
    ),
    (
        "Agrupar o mesmo fato",
        "Deduplicação e agrupamento por similaridade.",
    ),
    (
        "Ler em profundidade",
        "Texto completo das matérias mais relevantes.",
    ),
    (
        "Redigir o relance",
        "Síntese com citações e contraste de enquadramento.",
    ),
    (
        "Conferir citações",
        "Referências, números e limites da amostra.",
    ),
];

#[derive(Properties, PartialEq)]
struct LoadingProps {
    pub query: String,
    pub job: Option<JobView>,
}

#[function_component(LoadingView)]
fn loading_view(props: &LoadingProps) -> Html {
    let job = props.job.as_ref();
    let status = job.map(|value| value.status.as_str()).unwrap_or("queued");
    let done = status == "done";
    let step = job.map(|value| value.step).unwrap_or(0);
    let progress = job
        .map(|value| value.progress)
        .unwrap_or(0.0)
        .clamp(0.0, 100.0);
    let percent = format!("{progress:.0}");
    let detail = job
        .map(|value| value.detail.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Preparando a busca e organizando as fontes.".to_string());
    let eta = job.and_then(|value| value.eta_seconds);
    let eta_label = if done {
        "concluído".to_string()
    } else if status == "queued" {
        "na fila".to_string()
    } else {
        match eta {
            Some(0..=2) => "quase lá".to_string(),
            Some(seconds) if seconds < 60 => format!("~{seconds}s restantes"),
            Some(seconds) => format!("~{} min restantes", (seconds as f64 / 60.0).ceil() as u64),
            None => "calculando o tempo".to_string(),
        }
    };
    let stage_label = if done {
        "Relance pronto".to_string()
    } else if status == "queued" {
        "Aguardando uma vaga".to_string()
    } else {
        format!("Etapa {} de 6", step.clamp(1, 6))
    };
    let counts = job.and_then(|value| match (value.current, value.total) {
        (Some(current), Some(total)) if total > 0 => Some(format!("{current} de {total} matérias")),
        _ => None,
    });
    let read_count = job
        .and_then(|value| value.current)
        .filter(|_| step == 4)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "—".to_string());

    use_effect_with((), |_| {
        focus_selector(".loading-title");
        || ()
    });

    html! {
        <main class="loading-page" aria-busy="true">
            <section class="loading-copy">
                <p class="hero-kicker"><span class="status-dot"></span>{ "Relance em construção" }</p>
                <h1 class="loading-title">{ format!("“{}”", sentence_case(&props.query)) }</h1>
                <p class="loading-detail" aria-live="polite" aria-atomic="false">{ detail }</p>

                <div class="loading-meter" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={percent.clone()} aria-label="Progresso do relance">
                    <span class="loading-meter-fill" style={format!("width:{progress:.1}%")}></span>
                </div>
                <div class="loading-meta">
                    <span class="loading-percent">{ format!("{percent}%") }</span>
                    <span>{ eta_label.clone() }</span>
                    <span class="loading-stage-label">{ stage_label }</span>
                </div>

                if let Some(counts) = &counts {
                    <p class="loading-counts"><span class="status-dot"></span>{ counts }</p>
                }

                <ol class="loading-steps">
                    { for LOADING_STEPS.iter().enumerate().map(|(index, (label, description))| {
                        let number = index + 1;
                        let state = if done || step > number {
                            "is-done"
                        } else if step == number || (step == 0 && number == 1) {
                            "is-active"
                        } else {
                            "is-pending"
                        };
                        let step_detail = if number == 4 {
                            counts.clone().unwrap_or_else(|| description.to_string())
                        } else {
                            description.to_string()
                        };
                        html! {
                            <li class={classes!("loading-step", state)}>
                                <span class="loading-step-marker" aria-hidden="true">
                                    { if state == "is-done" { icon("check") } else { html! { <span>{ number }</span> } } }
                                </span>
                                <span class="loading-step-copy">
                                    <strong>{ label }</strong>
                                    <small>{ step_detail }</small>
                                </span>
                                <span class="loading-step-state">{ if state == "is-done" { "feito" } else if state == "is-active" { "agora" } else { "" } }</span>
                            </li>
                        }
                    }) }
                </ol>
            </section>

            <aside class="loading-stage" aria-hidden="true">
                <div class="loading-sky">
                    <span class="loading-sun"></span>
                    <span class="loading-horizon"></span>
                    <span class="loading-scan"></span>
                    <span class="loading-star loading-star-a"></span>
                    <span class="loading-star loading-star-b"></span>
                    <span class="loading-star loading-star-c"></span>
                </div>
                <div class="loading-number">
                    <strong>{ percent.clone() }</strong><span>{ "%" }</span>
                </div>
                <p class="loading-stage-note">
                    { "Leitura seletiva: o algoritmo escolhe poucos artigos para abrir a íntegra e usa o restante como manchete e metadado." }
                </p>
                <dl class="loading-facts">
                    <div><dt>{ "Leituras" }</dt><dd>{ read_count }</dd></div>
                    <div><dt>{ "Restante" }</dt><dd>{ eta_label.clone() }</dd></div>
                    <div><dt>{ "Etapa" }</dt><dd>{ format!("{} / 6", step.clamp(1, 6)) }</dd></div>
                </dl>
                <div class="loading-paper" aria-hidden="true">
                    <span></span><span></span><span></span><span></span><span></span>
                </div>
            </aside>
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct ErrorProps {
    pub query: String,
    pub message: String,
    pub on_retry: Callback<()>,
    pub on_home: Callback<()>,
}

#[function_component(ErrorView)]
fn error_view(props: &ErrorProps) -> Html {
    let retry = {
        let on_retry = props.on_retry.clone();
        Callback::from(move |_| on_retry.emit(()))
    };
    let home = {
        let on_home = props.on_home.clone();
        Callback::from(move |_| on_home.emit(()))
    };
    html! {
        <main class="state-page">
            <div class="error-state">
                { icon("alert") }
                <h1>{ "Não foi possível montar o relance" }</h1>
                <p>{ &props.message }</p>
                <p class="chart-note">{ format!("Tema: “{}”", sentence_case(&props.query)) }</p>
                <div class="state-actions">
                    <button type="button" class="btn btn-primary" onclick={retry}>{ "Tentar de novo" }</button>
                    <button type="button" class="btn btn-ghost" onclick={home}>{ "Voltar ao início" }</button>
                </div>
            </div>
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct EmptyProps {
    pub query: String,
    pub on_home: Callback<()>,
}

#[function_component(EmptyView)]
fn empty_view(props: &EmptyProps) -> Html {
    let home = {
        let on_home = props.on_home.clone();
        Callback::from(move |_| on_home.emit(()))
    };
    html! {
        <main class="state-page">
            <div class="empty-state">
                <span class="empty-mark">{ "—" }</span>
                <h1>{ "Nenhuma manchete encontrada" }</h1>
                <p>{ format!("Não encontramos artigos suficientes para “{}” na janela de 7 dias. Tente um tema mais amplo ou outra grafia.", sentence_case(&props.query)) }</p>
                <button type="button" class="btn btn-secondary" onclick={home}>{ "Escolher outro tema" }</button>
            </div>
        </main>
    }
}

// ---------------------------------------------------------------- Saved / Sources

#[derive(Properties, PartialEq)]
struct SavedProps {
    pub items: Vec<web::SavedItem>,
    pub on_search: Callback<String>,
    pub on_remove: Callback<String>,
    pub on_home: Callback<()>,
}

#[function_component(SavedView)]
fn saved_view(props: &SavedProps) -> Html {
    let home = {
        let on_home = props.on_home.clone();
        Callback::from(move |_| on_home.emit(()))
    };
    html! {
        <main class="index-page">
            <nav class="crumbs" aria-label="Trilha"><span>{ "Salvos" }</span></nav>
            <header class="index-head">
                <p class="micro-label">{ "Sua lista" }</p>
                <h1>{ "Relances salvos." }</h1>
                <p>{ "Tudo fica no seu navegador. Nada é enviado para uma conta." }</p>
            </header>
            if props.items.is_empty() {
                <div class="empty-state">
                    <span class="empty-mark">{ "—" }</span>
                    <h2>{ "Nada salvo ainda" }</h2>
                    <p>{ "Abra um relance e use “Salvar” para encontrá-lo aqui." }</p>
                    <button type="button" class="btn btn-secondary" onclick={home}>{ "Explorar um tema" }</button>
                </div>
            } else {
                <div class="saved-list">
                    { for props.items.iter().map(|item| {
                        let query = item.query.clone();
                        let on_search = props.on_search.clone();
                        let onclick = Callback::from(move |_| on_search.emit(query.clone()));
                        let query_remove = item.query.clone();
                        let on_remove = props.on_remove.clone();
                        let remove = Callback::from(move |_| on_remove.emit(query_remove.clone()));
                        html! {
                            <div class="saved-row">
                                <button type="button" class="saved-main" onclick={onclick}>
                                    <span>{ sentence_case(&item.query) }</span>
                                    <small>{ format!("Salvo em {}", web::format_date(&item.saved_at)) }</small>
                                </button>
                                <button type="button" class="icon-btn" onclick={remove} aria-label={format!("Remover {}", item.query)}>{ icon("trash") }</button>
                            </div>
                        }
                    }) }
                </div>
            }
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct SourcesProps {
    pub sources: Vec<SourceInfo>,
    pub on_home: Callback<()>,
}

#[function_component(SourcesView)]
fn sources_view(props: &SourcesProps) -> Html {
    let home = {
        let on_home = props.on_home.clone();
        Callback::from(move |_| on_home.emit(()))
    };
    html! {
        <main class="index-page">
            <nav class="crumbs" aria-label="Trilha"><span>{ "Fontes" }</span></nav>
            <header class="index-head">
                <p class="micro-label">{ "Metodologia" }</p>
                <h1>{ "Quem está na amostra." }</h1>
                <p>{ "A curadoria de demonstração cobre veículos nacionais, regionais, especializados e internacionais. Fontes oficiais, empresas, associações e blogs sem linha editorial conhecida permanecem “Sem avaliação”: não atribuímos espectro sem base. Em produção, esta camada é plugável: AllSides, Ad Fontes, MBFC ou uma base própria auditável." }</p>
            </header>
            <div class="source-table">
                <div class="source-head source-head-sources" aria-hidden="true">
                    <span>{ "Veículo" }</span><span>{ "Viés" }</span><span>{ "Factualidade" }</span><span>{ "Ownership" }</span>
                </div>
                { for props.sources.iter().map(|source| {
                    html! {
                        <div class="source-row source-row-sources">
                            <div class="source-name">
                                <a class="source-link-name" href={format!("https://{}", source.domain)} target="_blank" rel="noopener noreferrer">{ &source.name }</a>
                                <span>{ &source.domain }</span>
                            </div>
                            { tag(source.lean) }
                            { fact_tag(source.factuality) }
                            <span class="source-ownership">{ source.ownership.clone().unwrap_or_else(|| "—".to_string()) }</span>
                        </div>
                    }
                }) }
            </div>
            <p class="method-note">{ "A avaliação é uma curadoria editorial de demonstração. Não é uma certificação de verdade, qualidade ou intenção. O Mirante mostra a fonte de cada avaliação e aceita contestação." }</p>
            <button type="button" class="btn btn-secondary" onclick={home}>{ "Voltar ao início" }</button>
        </main>
    }
}

// ---------------------------------------------------------------- App

#[function_component(App)]
pub fn app() -> Html {
    let view = use_state(|| View::Home);
    let topics = use_state(default_topics);
    let theme = use_state(web::get_theme);
    let toast = use_state(|| Option::<String>::None);
    let saved = use_state(web::saved_items);

    let show_toast = {
        let toast = toast.clone();
        Callback::from(move |message: String| {
            toast.set(Some(message.clone()));
            let toast = toast.clone();
            Timeout::new(2400, move || toast.set(None)).forget();
        })
    };

    let run_search = {
        let view = view.clone();
        let toast = toast.clone();
        Callback::from(move |query: String| {
            let query = query.trim().to_string();
            if query.is_empty() {
                return;
            }
            web::push_query(&query);
            view.set(View::Loading {
                query: query.clone(),
                job: None,
            });
            let view = view.clone();
            let toast = toast.clone();
            spawn_local(async move {
                let mut job = match api::start_job(&query, 7).await {
                    Ok(job) => job,
                    Err(message) => {
                        toast.set(Some(message.clone()));
                        view.set(View::Error { query, message });
                        return;
                    }
                };
                loop {
                    view.set(View::Loading {
                        query: query.clone(),
                        job: Some(job.clone()),
                    });
                    if job.status == "done" {
                        if let Some(analysis) = job.analysis.clone() {
                            if analysis.stats.total_articles == 0 {
                                view.set(View::Empty { query });
                            } else {
                                view.set(View::Report(analysis));
                            }
                        } else {
                            let message =
                                "O relance terminou sem conteúdo. Tente novamente.".to_string();
                            toast.set(Some(message.clone()));
                            view.set(View::Error { query, message });
                        }
                        break;
                    }
                    if job.status == "error" {
                        let message = job
                            .error
                            .clone()
                            .unwrap_or_else(|| "A análise falhou no servidor.".to_string());
                        toast.set(Some(message.clone()));
                        view.set(View::Error { query, message });
                        break;
                    }
                    TimeoutFuture::new(450).await;
                    match api::get_job(&job.id).await {
                        Ok(next) => job = next,
                        Err(message) => {
                            toast.set(Some(message.clone()));
                            view.set(View::Error { query, message });
                            break;
                        }
                    }
                }
            });
        })
    };

    let home = {
        let view = view.clone();
        Callback::from(move |_| {
            if let Some(window) = web_sys::window() {
                let _ = window.history().and_then(|history| {
                    history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some("/"))
                });
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
            view.set(View::Home);
        })
    };

    let open_saved = {
        let view = view.clone();
        Callback::from(move |_| {
            if let Some(window) = web_sys::window() {
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
            view.set(View::Saved);
        })
    };

    let open_sources = {
        let view = view.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            view.set(View::SourcesLoading);
            let view = view.clone();
            let toast = toast.clone();
            spawn_local(async move {
                match api::sources().await {
                    Ok(sources) => view.set(View::Sources(sources)),
                    Err(message) => {
                        toast.set(Some(message.clone()));
                        view.set(View::SourcesError(message));
                    }
                }
            });
        })
    };

    let toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let next = if *theme == "dark" { "light" } else { "dark" };
            web::set_theme(next);
            theme.set(next.to_string());
        })
    };

    // Initial mount: theme + URL + topics.
    {
        let run_search = run_search.clone();
        let topics = topics.clone();
        use_effect_with((), move |_| {
            web::set_theme(&web::get_theme());
            if let Some(query) = web::read_query_param() {
                run_search.emit(query);
            }
            spawn_local(async move {
                if let Ok(items) = api::topics().await {
                    if !items.is_empty() {
                        topics.set(items);
                    }
                }
            });
            || ()
        });
    }

    let on_save = {
        let saved = saved.clone();
        let show_toast = show_toast.clone();
        let view = view.clone();
        Callback::from(move |_| {
            if let View::Report(analysis) = &*view {
                let is_now_saved = web::toggle_saved(&analysis.query, &analysis.query);
                saved.set(web::saved_items());
                show_toast.emit(if is_now_saved {
                    "Relance salvo".to_string()
                } else {
                    "Removido dos salvos".to_string()
                });
            }
        })
    };

    let on_share = {
        let show_toast = show_toast.clone();
        Callback::from(move |_| {
            let show_toast = show_toast.clone();
            if let Some(window) = web_sys::window() {
                if let Ok(href) = window.location().href() {
                    spawn_local(async move {
                        match web::copy(&href).await {
                            Ok(()) => show_toast.emit("Link copiado".to_string()),
                            Err(error) => show_toast.emit(error),
                        }
                    });
                }
            }
        })
    };

    let on_remove = {
        let saved = saved.clone();
        let show_toast = show_toast.clone();
        Callback::from(move |query: String| {
            web::remove_saved(&query);
            saved.set(web::saved_items());
            show_toast.emit("Removido dos salvos".to_string());
        })
    };

    let retry = {
        let view = view.clone();
        let run_search = run_search.clone();
        Callback::from(move |_| {
            if let View::Error { query, .. } = &*view {
                run_search.emit(query.clone());
            }
        })
    };

    let open_articles = {
        let view = view.clone();
        Callback::from(move |selection: Vec<Lean>| {
            if let View::Report(analysis) = &*view {
                if let Some(window) = web_sys::window() {
                    window.scroll_to_with_x_and_y(0.0, 0.0);
                }
                view.set(View::Articles {
                    analysis: analysis.clone(),
                    selected: selection,
                });
            }
        })
    };

    let back_to_report = {
        let view = view.clone();
        Callback::from(move |_| {
            if let View::Articles { analysis, .. } = &*view {
                if let Some(window) = web_sys::window() {
                    window.scroll_to_with_x_and_y(0.0, 0.0);
                }
                view.set(View::Report(analysis.clone()));
            }
        })
    };

    let current_query = match &*view {
        View::Loading { query, .. } => query.clone(),
        View::Empty { query } => query.clone(),
        View::Report(analysis) => analysis.query.clone(),
        View::Articles { analysis, .. } => analysis.query.clone(),
        View::Error { query, .. } => query.clone(),
        _ => String::new(),
    };
    let active = match &*view {
        View::Saved => "saved",
        View::Sources(_) | View::SourcesLoading | View::SourcesError(_) => "sources",
        _ => "home",
    };
    let is_saved = match &*view {
        View::Report(analysis) => saved.iter().any(|item| item.query == analysis.query),
        _ => false,
    };

    let content = match &*view {
        View::Home => html! {
            <Home topics={(*topics).clone()} on_search={run_search.clone()} on_sources={open_sources.clone()} />
        },
        View::Loading { query, job } => {
            html! { <LoadingView query={query.clone()} job={job.clone()} /> }
        }
        View::Empty { query } => {
            html! { <EmptyView query={query.clone()} on_home={home.clone()} /> }
        }
        View::Report(analysis) => html! {
            <ReportView analysis={analysis.clone()} saved={is_saved} on_save={on_save.clone()} on_share={on_share.clone()} on_retry={retry.clone()} on_open_articles={open_articles.clone()} />
        },
        View::Articles { analysis, selected } => html! {
            <AllArticlesView analysis={analysis.clone()} selected={selected.clone()} on_back={back_to_report.clone()} />
        },
        View::Error { query, message } => html! {
            <ErrorView query={query.clone()} message={message.clone()} on_retry={retry.clone()} on_home={home.clone()} />
        },
        View::Saved => html! {
            <SavedView items={(*saved).clone()} on_search={run_search.clone()} on_remove={on_remove.clone()} on_home={home.clone()} />
        },
        View::SourcesLoading => html! {
            <main class="state-page"><div class="loading-inline">{ "Carregando fontes…" }</div></main>
        },
        View::Sources(sources) => html! {
            <SourcesView sources={sources.clone()} on_home={home.clone()} />
        },
        View::SourcesError(message) => html! {
            <main class="state-page"><div class="error-state">{ icon("alert") }<h1>{ "Não foi possível carregar as fontes" }</h1><p>{ message.clone() }</p><button type="button" class="btn btn-secondary" onclick={Callback::from({ let home = home.clone(); move |_| home.emit(()) })}>{ "Voltar" }</button></div></main>
        },
    };

    let toast_text = (*toast).clone().unwrap_or_default();

    html! {
        <>
            <Header
                on_search={run_search.clone()}
                on_home={home.clone()}
                on_saved={open_saved.clone()}
                on_sources={open_sources.clone()}
                on_theme={toggle_theme}
                theme={(*theme).clone()}
                current_query={current_query}
                active={active.to_string()}
            />
            { content }
            <footer class="app-footer">
                <div><strong>{ "Mirante" }</strong><span>{ "A notícia por inteiro." }</span></div>
                <div class="app-footer-links">
                    <button type="button" onclick={Callback::from({ let open_sources = open_sources.clone(); move |_| open_sources.emit(()) })}>{ "Fontes e metodologia" }</button>
                    <span>{ "Rust + WebAssembly · 2026" }</span>
                </div>
            </footer>
            <div class={classes!("toast")} data-show={toast.is_some().to_string()}>{ toast_text }</div>
        </>
    }
}

fn default_topics() -> Vec<Topic> {
    vec![
        Topic {
            label: "Reforma tributária".into(),
            query: "reforma tributária".into(),
            description: "O que muda, quem paga e o que ficou fora do texto.".into(),
        },
        Topic {
            label: "Regulação da IA".into(),
            query: "regulação da inteligência artificial".into(),
            description: "Direitos autorais, supervisão e a corrida por infraestrutura.".into(),
        },
        Topic {
            label: "Eleições 2026".into(),
            query: "eleições 2026".into(),
            description: "Pesquisas, alianças e a agenda que cada campo escolhe pautar.".into(),
        },
        Topic {
            label: "Tarifa de energia".into(),
            query: "tarifa de energia".into(),
            description: "Reajuste, transmissão e o impacto na conta de quem menos consome.".into(),
        },
        Topic {
            label: "Data centers".into(),
            query: "data centers".into(),
            description: "Energia, água e território na expansão da infraestrutura digital.".into(),
        },
        Topic {
            label: "Segurança pública".into(),
            query: "segurança pública".into(),
            description: "Operações, letalidade e o debate sobre política de drogas.".into(),
        },
        Topic {
            label: "Clima extremo".into(),
            query: "mudança climática eventos extremos".into(),
            description: "Enchentes, calor e adaptação nas cidades.".into(),
        },
        Topic {
            label: "Mercado de trabalho".into(),
            query: "mercado de trabalho emprego".into(),
            description: "Informalidade, renda e as vagas que a cobertura não mostra.".into(),
        },
    ]
}
