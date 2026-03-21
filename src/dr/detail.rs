use chrono::NaiveDate;
use serde_json::{Value, json};
use tracing::info;

use crate::error::{LauyerError, Result};

use super::session::DrSession;

const DETAIL_URL: &str = "https://diariodarepublica.pt/dr/screenservices/dr/Legislacao_Conteudos/Conteudo_Detalhe/DataActionGetConteudoDataAndApplicationSettings";

/// API version hash for the detail endpoint (different from search).
const DETAIL_API_VERSION: &str = "xbG2d6DNViOgTiMUtZPtGw";

// ---------------------------------------------------------------------------
// Result type
// ---------------------------------------------------------------------------

/// Full detail of a DR legislation act.
#[derive(Debug, Clone)]
pub struct DrDetailResult {
    pub id: String,
    pub titulo: String,
    pub numero: String,
    pub publicacao: String,
    pub sumario: String,
    pub texto: String,
    pub data_publicacao: Option<NaiveDate>,
    pub emissor: String,
    pub serie: String,
    pub tipo_diploma: String,
    pub vigencia: String,
    pub url_pdf: String,
    pub eli: String,
    pub notas: String,
    pub pagina: String,
    pub dr_url: String,
}

// ---------------------------------------------------------------------------
// Fetch
// ---------------------------------------------------------------------------

/// Fetch the full detail of a DR legislation act by its `ConteudoId`.
///
/// Requires `tipo` (e.g. "Portaria"), `numero` (e.g. "123-A/2026/1"),
/// and `year` (e.g. 2026) to construct the request key.
pub async fn fetch_detail(
    session: &DrSession,
    conteudo_id: &str,
    tipo: &str,
    numero: &str,
    year: u32,
) -> Result<DrDetailResult> {
    let body = build_detail_body(session, conteudo_id, tipo, numero, year);

    info!(conteudo_id, "Fetching DR document detail");

    let response = session
        .client()
        .inner()
        .post(DETAIL_URL)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("X-CSRFToken", session.csrf_token())
        .header("Accept", "application/json")
        .header("outsystems-locale", "pt-PT")
        .json(&body)
        .send()
        .await
        .map_err(|e| LauyerError::Http { source: e, url: DETAIL_URL.to_owned() })?;

    let status = response.status();
    if !status.is_success() {
        return Err(LauyerError::Session {
            message: format!("DR detail fetch returned HTTP {status}"),
        });
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| LauyerError::Http { source: e, url: DETAIL_URL.to_owned() })?;

    parse_detail_response(&response_text, conteudo_id, tipo)
}

// ---------------------------------------------------------------------------
// Body builder
// ---------------------------------------------------------------------------

/// Derive the slug used in the detail API `Key` and `Numero` fields.
///
/// Input: "123-A/2026/1" → ("123-a", 2026)
/// Input: "79-A/2026" → ("79-a", 2026)
pub fn derive_slug_and_year(numero: &str, fallback_year: u32) -> (String, u32) {
    let parts: Vec<&str> = numero.split('/').collect();
    let slug = parts.first().unwrap_or(&"").to_lowercase();
    let year = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(fallback_year);
    (slug, year)
}

/// Derive the tipo slug for the detail API URL.
///
/// "Portaria" → "portaria", "Decreto-Lei" → "decreto-lei"
fn tipo_slug(tipo: &str) -> String {
    tipo.to_lowercase()
}

pub fn build_detail_body(
    session: &DrSession,
    conteudo_id: &str,
    tipo: &str,
    numero: &str,
    year: u32,
) -> Value {
    let (numero_slug, derived_year) = derive_slug_and_year(numero, year);
    let t_slug = tipo_slug(tipo);
    let key = format!("{numero_slug}-{derived_year}-{conteudo_id}");

    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string();
    let guid = uuid::Uuid::new_v4().to_string();

    let version_info = json!({
        "moduleVersion": session.module_version(),
        "apiVersion": DETAIL_API_VERSION
    });

    let variables = build_screen_variables(conteudo_id, &numero_slug, derived_year, &t_slug, &key);

    let client_variables = build_client_variables(&today, &now, &guid);

    json!({
        "versionInfo": version_info,
        "viewName": "Legislacao_Conteudos.Conteudo_Detalhe",
        "screenData": { "variables": variables },
        "clientVariables": client_variables
    })
}

fn build_screen_variables(
    conteudo_id: &str,
    numero_slug: &str,
    year: u32,
    tipo_slug: &str,
    key: &str,
) -> Value {
    let mut vars = serde_json::Map::new();
    let s = |v: &str| Value::String(v.to_owned());

    vars.insert("ParteIdAux".into(), s("0"));
    vars.insert("DetalheConteudoElastic".into(), build_empty_es_template());
    vars.insert("IsRended".into(), Value::Bool(false));
    vars.insert("DiarioRepId".into(), s("0"));
    vars.insert("DipLegisId".into(), s(conteudo_id));
    vars.insert("DipDGOId".into(), s("0"));
    vars.insert("DipRegTrabId".into(), s("0"));
    vars.insert("DipLegacorId".into(), s("0"));
    vars.insert("DipDGAPId".into(), s("0"));
    vars.insert("ActSocId".into(), s("0"));
    vars.insert("AcSTADipId".into(), s("0"));
    vars.insert("ContPubId".into(), s("0"));
    vars.insert("DiplExtId".into(), s("0"));
    vars.insert("ConteudoId".into(), s(conteudo_id));
    vars.insert("Pesquisa".into(), s(""));
    vars.insert("Comes1".into(), s("Pesquisa"));
    vars.insert("Numero".into(), s(numero_slug));
    vars.insert("Year".into(), Value::Number(year.into()));
    vars.insert("length".into(), Value::Number(0.into()));
    vars.insert("ShowResumoPT".into(), Value::Bool(false));
    vars.insert("HasJurisprudenciaAssociadaVar".into(), Value::Bool(false));
    vars.insert("IsPageTracked".into(), Value::Bool(false));
    vars.insert("ShowDiplomaFragmentacaoIndice".into(), Value::Bool(false));
    vars.insert("ShowDiplomaFragmentacaoTextoCompleto".into(), Value::Bool(false));
    vars.insert("ShowDiplomaFragmentacaoMenuCabecalho".into(), Value::Bool(false));
    vars.insert("ShowDiplomaAtoOriginalTexto".into(), Value::Bool(true));
    vars.insert("ELI_HTML".into(), s(""));
    vars.insert("IsShowConteudoRelacionado".into(), Value::Bool(true));
    vars.insert("IsRefinarPesquisa".into(), Value::Bool(true));
    vars.insert("IsToHideElements".into(), Value::Bool(false));
    vars.insert("TextoFormatadoAux".into(), s(""));
    vars.insert("FragmentoVersaoId".into(), s("0"));
    vars.insert("IsPrint".into(), Value::Bool(false));
    vars.insert("IsLoadingPDFAtoOriginal".into(), Value::Bool(false));
    vars.insert("IsCopyTextLoading".into(), Value::Bool(false));
    vars.insert("ShowScrollButtons".into(), Value::Bool(true));
    vars.insert("KeyAux".into(), s(""));
    vars.insert("Tipo".into(), s(tipo_slug));
    vars.insert("_tipoInDataFetchStatus".into(), Value::Number(1.into()));
    vars.insert("Key".into(), s(key));
    vars.insert("_keyInDataFetchStatus".into(), Value::Number(1.into()));
    vars.insert("ParteId".into(), s("0"));
    vars.insert("_parteIdInDataFetchStatus".into(), Value::Number(1.into()));
    vars.insert("Emissor_Designacao".into(), s(""));
    vars.insert("_emissor_DesignacaoInDataFetchStatus".into(), Value::Number(1.into()));

    Value::Object(vars)
}

fn build_client_variables(today: &str, now: &str, guid: &str) -> Value {
    json!({
        "NewUser": "https://diariodarepublica.pt/dr/utilizador/registar",
        "PesquisaAvancada": "https://diariodarepublica.pt/dr/pesquisa-avancada",
        "Login": "https://diariodarepublica.pt/dr/utilizador/entrar",
        "Data": today,
        "DicionarioJuridicoId": "0",
        "FullHTMLURL_EN": "https://diariodarepublica.pt/dr/en",
        "StartIndex": 0,
        "DiarioRepublicaId": "",
        "Serie": true,
        "DiplomaConteudoId": "",
        "CookiePath": "/dr/",
        "Session_GUID": guid,
        "DateTime": now,
        "FullHTMLURL": "https://diariodarepublica.pt/dr/home",
        "TipoDeUtilizador": "",
        "GUID": ""
    })
}

/// Build a minimal empty ES response structure required by the `OutSystems` screen model.
fn build_empty_es_template() -> Value {
    let empty_agg = || {
        json!({
            "doc_count_error_upper_bound": "0",
            "sum_other_doc_count": "0",
            "buckets": {
                "List": [],
                "EmptyListItem": {"key": "", "doc_count": "0", "key_as_string": "", "isActive": false}
            }
        })
    };

    json!({
        "Took": "0",
        "Timed_out": false,
        "shards": {"Total": "0", "Successful": "0", "Skipped": "0", "Failed": "0"},
        "Hits": {
            "Total": {"Value": "0", "Relation": ""},
            "Max_score": "0",
            "Hits": {
                "List": [],
                "EmptyListItem": {
                    "index": "",
                    "type": "",
                    "id": "",
                    "score": "0",
                    "source": build_empty_es_source(),
                    "Highlight": {
                        "Title": {"List": [], "EmptyListItem": ""},
                        "Sumario": {"List": [], "EmptyListItem": ""},
                        "Designacao": {"List": [], "EmptyListItem": ""},
                        "Texto": {"List": [], "EmptyListItem": ""}
                    }
                }
            }
        },
        "aggregations": {
            "SerieAgg": empty_agg(),
            "TipoAtoAgg": empty_agg(),
            "TipoConteudoAgg": empty_agg(),
            "EntidadeEmitenteAgg": empty_agg(),
            "EntidadeProponenteAgg": empty_agg(),
            "EntidadePrincipalAgg": empty_agg(),
            "TipoConteudoAggOutros": empty_agg(),
            "EmissorAgg": empty_agg(),
            "DescritorAgg": empty_agg(),
            "ParteAgg": empty_agg(),
            "CalendarioAgg": empty_agg(),
            "JurisprudenciaAgg": empty_agg(),
            "JurisAggs": {
                "buckets": {
                    "List": [],
                    "EmptyListItem": {"key": "", "doc_count": "0", "key_as_string": "", "isActive": false}
                }
            },
            "VigenciaAgg": empty_agg(),
            "ConsolidacaoEstadoAgg": empty_agg(),
            "ConsolidacaoVisibilityAgg": empty_agg()
        }
    })
}

fn build_empty_es_source() -> Value {
    let empty_list = || json!({"List": [], "EmptyListItem": ""});
    let s = |v: &str| Value::String(v.to_owned());
    let mut m = serde_json::Map::new();

    // String fields with defaults
    let str_fields = [
        ("NumeroInt", "0"),
        ("DataPublicacaoAJ", "1900-01-01"),
        ("DocType", ""),
        ("Visibility", ""),
        ("Vigencia", ""),
        ("Title_bst_10k", ""),
        ("TextoEntradaVigor", ""),
        ("Type", ""),
        ("NumeroFonte", ""),
        ("Emissor", ""),
        ("Ano", ""),
        ("Paginas", ""),
        ("TipoAJ", ""),
        ("SerieNR", ""),
        ("Suplemento", ""),
        ("OrdemDR", ""),
        ("Texto", ""),
        ("ConteudoId", "0"),
        ("EntidadePrincipal", ""),
        ("DataPublicacao", "1900-01-01"),
        ("EntidadeEmitente", ""),
        ("DataDistribuicao", "1900-01-01"),
        ("PaginaFinal", "0"),
        ("Numero", ""),
        ("FileId", "0"),
        ("TipoConteudo", ""),
        ("EntidadeResponsavel", ""),
        ("NumeroAJ", ""),
        ("DbId", "0"),
        ("Thesaurus_descritor_eq", ""),
        ("DataAssinatura", "1900-01-01"),
        ("WhenSearchable", ""),
        ("Id", ""),
        ("Title", ""),
        ("Ordem", "0"),
        ("ConteudoTitle", ""),
        ("ClassName", ""),
        ("NumeroDR", ""),
        ("DataEntradaVigor", "1900-01-01T00:00:00"),
        ("PaginaInicial", "0"),
        ("Acronimo", ""),
        ("TextoAssociacao", ""),
        ("Serie", ""),
        ("CreationDate", "1900-01-01T00:00:00"),
        ("Descritor_texto", ""),
        ("ResumoAJ", ""),
        ("Sumario", ""),
        ("Views", "0"),
        ("Fonte", ""),
        ("Tipo", ""),
        ("ModificationDate", "1900-01-01T00:00:00"),
        ("EmissorEN", ""),
        ("Parte", ""),
        ("TipoEN", ""),
        ("TextoNota", ""),
        ("resumo", ""),
        ("resumoEN", ""),
        ("Designacao", ""),
        ("ConsolidacaoType", ""),
        ("DiplomaBase", ""),
        ("concelho", ""),
        ("empresa", ""),
        ("assunto", ""),
        ("consolidacaoEstado", ""),
        ("DiplomaFragmentadoId", ""),
        ("consolidacaoVisibility", ""),
        ("EntidadeProponente", ""),
    ];
    for (k, v) in str_fields {
        m.insert(k.into(), s(v));
    }

    // Bool fields
    let bool_fields = [
        ("Conteudo", false),
        ("AjPublica", false),
        ("Tratamento", false),
        ("Fragmentado", false),
        ("Regional", false),
        ("IsSelected", false),
        ("CPAlteracao", false),
    ];
    for (k, v) in bool_fields {
        m.insert(k.into(), Value::Bool(v));
    }

    // List fields
    let list_fields =
        ["TipoAssociacao", "Descritor", "Nota", "Thesaurus_descritor_np", "Thesaurus_descritor"];
    for k in list_fields {
        m.insert(k.into(), empty_list());
    }

    Value::Object(m)
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

pub fn parse_detail_response(
    response_text: &str,
    conteudo_id: &str,
    tipo: &str,
) -> Result<DrDetailResult> {
    let outer: Value = serde_json::from_str(response_text).map_err(|e| LauyerError::Parse {
        message: format!("Failed to parse DR detail response: {e}"),
        source_url: DETAIL_URL.to_owned(),
    })?;

    if let Some(exception) = outer.get("exception") {
        let msg = exception.get("message").and_then(Value::as_str).unwrap_or("Unknown exception");
        return Err(LauyerError::Session { message: format!("DR detail API exception: {msg}") });
    }

    let detail = outer.pointer("/data/DetalheConteudo").ok_or_else(|| LauyerError::Parse {
        message: "Missing 'data.DetalheConteudo' in DR detail response".to_owned(),
        source_url: DETAIL_URL.to_owned(),
    })?;

    let get =
        |key: &str| -> String { detail.get(key).and_then(Value::as_str).unwrap_or("").to_owned() };

    let data_publicacao = detail
        .get("DataPublicacao")
        .and_then(Value::as_str)
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let numero = get("Numero");
    let tipo_diploma = get("TipoDiploma");
    let tipo_slug_str = tipo_slug(if tipo_diploma.is_empty() { tipo } else { &tipo_diploma });
    let (numero_slug, year) = derive_slug_and_year(&numero, 0);
    let dr_url = format!(
        "https://diariodarepublica.pt/dr/detalhe/{tipo_slug_str}/{numero_slug}-{year}-{conteudo_id}"
    );

    info!(conteudo_id, titulo = get("Titulo"), "DR document detail fetched");

    Ok(DrDetailResult {
        id: get("Id"),
        titulo: get("Titulo"),
        numero,
        publicacao: get("Publicacao"),
        sumario: get("Sumario"),
        texto: get("Texto"),
        data_publicacao,
        emissor: get("Emissor"),
        serie: get("Serie"),
        tipo_diploma,
        vigencia: get("Vigencia"),
        url_pdf: get("URL_PDF"),
        eli: get("ELI"),
        notas: get("Notas"),
        pagina: get("Pagina"),
        dr_url,
    })
}
