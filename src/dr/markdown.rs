use crate::compact::strip_html_tags;
use crate::format::Renderable;

use super::detail::DrDetailResult;
use super::search::DrSearchResult;

impl Renderable for DrSearchResult {
    fn to_markdown(&self) -> String {
        let date_str = self
            .data_publicacao
            .map_or_else(|| "s/d".to_owned(), |d| d.format("%Y-%m-%d").to_string());

        let clean_sumario = strip_html_tags(&self.sumario);

        format!(
            "### {} n.º {} ({})\n**Emissor:** {}\n**Sumário:** {}",
            self.tipo, self.numero, date_str, self.emissor, clean_sumario
        )
    }

    fn to_json(&self) -> serde_json::Value {
        let date_str = self.data_publicacao.map_or_else(
            || serde_json::Value::Null,
            |d| serde_json::Value::String(d.format("%Y-%m-%d").to_string()),
        );

        serde_json::json!({
            "title": self.title,
            "tipo": self.tipo,
            "numero": self.numero,
            "data_publicacao": date_str,
            "emissor": self.emissor,
            "sumario": strip_html_tags(&self.sumario),
            "serie": self.serie,
            "db_id": self.db_id,
            "file_id": self.file_id,
            "tipo_conteudo": self.tipo_conteudo,
            "ano": self.ano,
            "conteudo_id": self.conteudo_id,
        })
    }

    fn table_row(&self) -> Option<(Vec<&str>, Vec<String>)> {
        let date_str = self
            .data_publicacao
            .map_or_else(|| "s/d".to_owned(), |d| d.format("%Y-%m-%d").to_string());

        let clean_sumario = strip_html_tags(&self.sumario);
        let truncated_sumario = if clean_sumario.len() > 60 {
            let mut end = 57;
            while end > 0 && !clean_sumario.is_char_boundary(end) {
                end -= 1;
            }
            let mut s = clean_sumario[..end].to_owned();
            s.push_str("...");
            s
        } else {
            clean_sumario
        };

        Some((
            vec!["Date", "Tipo", "Número", "Emissor", "Sumário"],
            vec![
                date_str,
                self.tipo.clone(),
                self.numero.clone(),
                self.emissor.clone(),
                truncated_sumario,
            ],
        ))
    }
}

impl Renderable for DrDetailResult {
    fn to_markdown(&self) -> String {
        let date_str = self
            .data_publicacao
            .map_or_else(|| "s/d".to_owned(), |d| d.format("%Y-%m-%d").to_string());

        let mut md = format!(
            "### {} n.º {} ({})\n**Emissor:** {}\n**Sumário:** {}\n",
            self.tipo_diploma, self.numero, date_str, self.emissor, self.sumario
        );

        if !self.url_pdf.is_empty() {
            md.push_str(&format!("**PDF:** {}\n", self.url_pdf));
        }
        if !self.eli.is_empty() {
            md.push_str(&format!("**ELI:** {}\n", self.eli));
        }
        if !self.dr_url.is_empty() {
            md.push_str(&format!("**DR:** {}\n", self.dr_url));
        }

        md.push_str("\n> Note: This text was extracted automatically and may contain interpretation errors. Always verify against the official source above.\n");

        if !self.texto.is_empty() {
            md.push_str(&format!("\n---\n\n{}", self.texto));
        }

        md
    }

    fn to_json(&self) -> serde_json::Value {
        let date_str = self.data_publicacao.map_or_else(
            || serde_json::Value::Null,
            |d| serde_json::Value::String(d.format("%Y-%m-%d").to_string()),
        );

        serde_json::json!({
            "id": self.id,
            "titulo": self.titulo,
            "numero": self.numero,
            "publicacao": self.publicacao,
            "sumario": self.sumario,
            "texto": self.texto,
            "data_publicacao": date_str,
            "emissor": self.emissor,
            "serie": self.serie,
            "tipo_diploma": self.tipo_diploma,
            "vigencia": self.vigencia,
            "url_pdf": self.url_pdf,
            "eli": self.eli,
            "notas": self.notas,
            "pagina": self.pagina,
            "dr_url": self.dr_url,
            "_disclaimer": "This text was extracted automatically and may contain interpretation errors. Always verify against the official DR source.",
        })
    }

    fn table_row(&self) -> Option<(Vec<&str>, Vec<String>)> {
        let date_str = self
            .data_publicacao
            .map_or_else(|| "s/d".to_owned(), |d| d.format("%Y-%m-%d").to_string());

        let truncated_texto = if self.texto.len() > 80 {
            let mut end = 77;
            while end > 0 && !self.texto.is_char_boundary(end) {
                end -= 1;
            }
            let mut s = self.texto[..end].to_owned();
            s.push_str("...");
            s
        } else {
            self.texto.clone()
        };

        Some((
            vec!["Date", "Tipo", "Número", "Emissor", "Texto"],
            vec![
                date_str,
                self.tipo_diploma.clone(),
                self.numero.clone(),
                self.emissor.clone(),
                truncated_texto,
            ],
        ))
    }
}
