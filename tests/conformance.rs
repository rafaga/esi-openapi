//! Checks the implemented endpoints against the OpenAPI fixture in
//! `resources/test/openapi.json`: every operation is implemented, and the HTTP
//! method, path placeholders and query keys used by each implementation match the spec.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Operations deliberately left unimplemented. Keep it empty unless there is a reason.
const NOT_IMPLEMENTED: &[&str] = &[];

const HTTP_METHODS: [&str; 5] = ["get", "post", "put", "delete", "patch"];

/// What the spec says about one operation.
struct SpecOperation {
    method: String,
    path: String,
    query_params: BTreeSet<String>,
    /// Properties of the (array item or object) response schema, with whether each is required.
    response_fields: Option<BTreeMap<String, bool>>,
    /// For a cursor page (`cursor` plus one array of records): the fields of a record.
    page_item_fields: Option<BTreeMap<String, bool>>,
}

/// One use of an operation ID in the sources.
struct Usage {
    file: String,
    op_id: String,
    /// HTTP method, when the source makes it explicit.
    method: Option<String>,
    /// Placeholders (`{x}`), when the source is a macro call.
    placeholders: Option<BTreeSet<String>>,
    /// Query keys, when the source is a macro call.
    query_keys: Option<BTreeSet<String>>,
    /// Name of the Rust type the macro call returns, without `Vec<>`.
    return_type: Option<String>,
    /// The `T` of a returned `Page<T>`.
    page_item: Option<String>,
}

/// A struct declared in the sources: its serialized field names.
struct RustStruct {
    fields: BTreeSet<String>,
}

struct Conformance {
    operations: BTreeMap<String, SpecOperation>,
    usages: Vec<Usage>,
    structs: BTreeMap<String, RustStruct>,
}

impl Conformance {
    fn load() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let spec: Value = serde_json::from_str(
            &fs::read_to_string(root.join("resources/test/openapi.json")).unwrap(),
        )
        .unwrap();
        let mut conformance = Conformance {
            operations: BTreeMap::new(),
            usages: Vec::new(),
            structs: BTreeMap::new(),
        };
        conformance.read_spec(&spec);
        let mut files: Vec<_> = fs::read_dir(root.join("src/groups"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "rs"))
            .collect();
        files.sort();
        for file in files {
            let source = fs::read_to_string(&file).unwrap();
            let name = file.file_name().unwrap().to_string_lossy().into_owned();
            conformance.read_structs(&source);
            conformance.read_source(&name, &source);
        }
        conformance
    }

    fn resolve<'a>(spec: &'a Value, node: &'a Value) -> &'a Value {
        match node.get("$ref").and_then(Value::as_str) {
            Some(reference) => {
                let mut current = spec;
                for key in reference.trim_start_matches("#/").split('/') {
                    current = &current[key];
                }
                Self::resolve(spec, current)
            }
            None => node,
        }
    }

    fn read_spec(&mut self, spec: &Value) {
        for (path, item) in spec["paths"].as_object().unwrap() {
            for method in HTTP_METHODS {
                let Some(operation) = item.get(method) else {
                    continue;
                };
                let op_id = operation["operationId"].as_str().unwrap().to_owned();
                let mut query_params = BTreeSet::new();
                let shared = item.get("parameters").and_then(Value::as_array);
                let own = operation.get("parameters").and_then(Value::as_array);
                for param in shared
                    .into_iter()
                    .flatten()
                    .chain(own.into_iter().flatten())
                {
                    let param = Self::resolve(spec, param);
                    if param["in"] == "query" {
                        query_params.insert(param["name"].as_str().unwrap().to_owned());
                    }
                }
                let response_fields = Self::response_fields(spec, operation);
                let page_item_fields = Self::page_item_fields(spec, operation);
                self.operations.insert(
                    op_id,
                    SpecOperation {
                        method: method.to_uppercase(),
                        path: path.clone(),
                        query_params,
                        response_fields,
                        page_item_fields,
                    },
                );
            }
        }
    }

    fn response_fields(spec: &Value, operation: &Value) -> Option<BTreeMap<String, bool>> {
        let ok = operation["responses"]
            .as_object()?
            .iter()
            .find(|(code, _)| code.starts_with('2'))?
            .1;
        let ok = Self::resolve(spec, ok);
        let schema = ok.get("content")?.get("application/json")?.get("schema")?;
        let mut schema = Self::resolve(spec, schema);
        if schema["type"] == "array" {
            schema = Self::resolve(spec, schema.get("items")?);
        }
        let properties = schema.get("properties")?.as_object()?;
        let required: BTreeSet<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|r| r.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        Some(
            properties
                .keys()
                .map(|k| (k.clone(), required.contains(k.as_str())))
                .collect(),
        )
    }

    /// Fields of a record in a cursor page: the response has a `cursor` property and
    /// one array property that holds the records.
    fn page_item_fields(spec: &Value, operation: &Value) -> Option<BTreeMap<String, bool>> {
        let ok = operation["responses"]
            .as_object()?
            .iter()
            .find(|(code, _)| code.starts_with('2'))?
            .1;
        let ok = Self::resolve(spec, ok);
        let schema = ok.get("content")?.get("application/json")?.get("schema")?;
        let schema = Self::resolve(spec, schema);
        let properties = schema.get("properties")?.as_object()?;
        properties.get("cursor")?;
        let mut arrays = properties
            .values()
            .map(|p| Self::resolve(spec, p))
            .filter(|p| p["type"] == "array");
        let records = Self::resolve(spec, arrays.next()?.get("items")?);
        if arrays.next().is_some() {
            return None;
        }
        let required: BTreeSet<&str> = records
            .get("required")
            .and_then(Value::as_array)
            .map(|r| r.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        Some(
            records
                .get("properties")?
                .as_object()?
                .keys()
                .map(|k| (k.clone(), required.contains(k.as_str())))
                .collect(),
        )
    }

    /// Collect the structs declared in a source file.
    fn read_structs(&mut self, source: &str) {
        let mut rest = source;
        while let Some(start) = rest.find("pub struct ") {
            let after = &rest[start + 11..];
            rest = after;
            let name: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let Some(body_start) = after.find('{') else {
                continue;
            };
            // Only structs with named fields: the `{` must come before any `;` or `(`.
            let header = &after[..body_start];
            if header.contains(';') || header.contains('(') {
                continue;
            }
            let Some(body_end) = after.find("\n}") else {
                continue;
            };
            let body = &after[body_start..body_end];
            let mut fields = BTreeSet::new();
            let mut rename: Option<String> = None;
            for line in body.lines().map(str::trim) {
                if let Some(attr) = line.strip_prefix("#[serde(rename = \"") {
                    rename = attr.split('"').next().map(str::to_owned);
                } else if let Some(field) = line.strip_prefix("pub ") {
                    if let Some(name) = field.split(':').next() {
                        fields.insert(rename.take().unwrap_or_else(|| name.trim().to_owned()));
                    }
                }
            }
            self.structs.insert(name, RustStruct { fields });
        }
    }

    /// Every double-quoted literal in `text`, in order.
    fn literals(text: &str) -> Vec<&str> {
        text.split('"').skip(1).step_by(2).collect()
    }

    fn read_source(&mut self, file: &str, source: &str) {
        // Macro calls: from `api_xxx!(` to the closing `);` at the start of a line.
        let mut rest = source;
        while let Some(start) = rest.find("api_") {
            let after = &rest[start..];
            let Some(open) = after.find("!(") else { break };
            let name = &after[..open];
            let is_macro = matches!(name, "api_get" | "api_post" | "api_put" | "api_delete");
            let end = after.find("\n    );").map_or(after.len(), |e| e + 6);
            if is_macro {
                self.read_macro(file, &name[4..], &after[open..end]);
            }
            rest = &after[(open + 2).min(after.len())..];
        }
        // Hand-written methods: `get_endpoint_for_op_id("X")` and the next HTTP method literal.
        let mut rest = source;
        while let Some(start) = rest.find("get_endpoint_for_op_id(\"") {
            let after = &rest[start + 24..];
            let op_id = after[..after.find('"').unwrap()].to_owned();
            let window = &after[..after.len().min(700)];
            let method = Self::literals(window)
                .into_iter()
                .find(|l| ["GET", "POST", "PUT", "DELETE"].contains(l))
                .map(str::to_owned);
            self.usages.push(Usage {
                file: file.to_owned(),
                op_id,
                method,
                placeholders: None,
                query_keys: None,
                return_type: None,
                page_item: None,
            });
            rest = after;
        }
    }

    fn read_macro(&mut self, file: &str, method: &str, call: &str) {
        let literals = Self::literals(call);
        let Some(op_id) = literals
            .iter()
            .find(|l| self.operations.contains_key(**l))
            .map(|l| (*l).to_owned())
        else {
            return;
        };
        let mut placeholders = BTreeSet::new();
        let mut query_keys = BTreeSet::new();
        for literal in literals.iter().filter(|l| **l != op_id) {
            if literal.starts_with('{') && literal.ends_with('}') {
                placeholders.insert(literal[1..literal.len() - 1].to_owned());
            } else if literal
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c == '-')
                && !literal.is_empty()
            {
                query_keys.insert((*literal).to_owned());
            }
        }
        self.usages.push(Usage {
            file: file.to_owned(),
            op_id,
            method: Some(method.to_uppercase()),
            placeholders: Some(placeholders),
            query_keys: Some(query_keys),
            return_type: Self::return_type(call),
            page_item: Self::page_item(call),
        });
    }

    /// The `T` of `Page<T>` when that is what a macro call returns.
    fn page_item(call: &str) -> Option<String> {
        let after = &call[call.find("RequestType::")?..];
        let after = &after[after.find(',')? + 1..];
        let inner = after.trim_start().strip_prefix("Page<")?;
        let name: String = inner
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        (!name.is_empty()).then_some(name)
    }

    /// The type after `RequestType::X,` in a macro call, without `Vec<>`.
    fn return_type(call: &str) -> Option<String> {
        let after = &call[call.find("RequestType::")?..];
        let after = &after[after.find(',')? + 1..];
        let ty = after.trim_start();
        let ty = ty.strip_prefix("Vec<").unwrap_or(ty);
        let name: String = ty
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        (!name.is_empty()).then_some(name)
    }

    fn path_placeholders(path: &str) -> BTreeSet<String> {
        path.split('{')
            .skip(1)
            .filter_map(|s| s.split('}').next())
            .map(str::to_owned)
            .collect()
    }
}

#[test]
fn every_operation_is_implemented() {
    let conformance = Conformance::load();
    let used: BTreeSet<&str> = conformance
        .usages
        .iter()
        .map(|u| u.op_id.as_str())
        .collect();
    let missing: Vec<&String> = conformance
        .operations
        .keys()
        .filter(|id| !used.contains(id.as_str()) && !NOT_IMPLEMENTED.contains(&id.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "operations without implementation: {missing:?}"
    );
}

#[test]
fn the_scan_finds_the_macro_calls() {
    // Guards the other checks against silently scanning nothing.
    let conformance = Conformance::load();
    let macros = conformance
        .usages
        .iter()
        .filter(|u| u.placeholders.is_some())
        .count();
    assert!(macros > 150, "only {macros} macro calls were found");
}

#[test]
fn every_used_operation_exists_in_the_spec() {
    let conformance = Conformance::load();
    let unknown: Vec<String> = conformance
        .usages
        .iter()
        .filter(|u| !conformance.operations.contains_key(&u.op_id))
        .map(|u| format!("{} ({})", u.op_id, u.file))
        .collect();
    assert!(unknown.is_empty(), "unknown operation IDs: {unknown:?}");
}

#[test]
fn http_methods_match_the_spec() {
    let conformance = Conformance::load();
    let mut wrong = Vec::new();
    for usage in &conformance.usages {
        let (Some(operation), Some(method)) = (
            conformance.operations.get(&usage.op_id),
            usage.method.as_ref(),
        ) else {
            continue;
        };
        if &operation.method != method {
            wrong.push(format!(
                "{} in {}: code uses {method}, spec says {}",
                usage.op_id, usage.file, operation.method
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn path_placeholders_match_the_spec() {
    let conformance = Conformance::load();
    let mut wrong = Vec::new();
    for usage in &conformance.usages {
        let (Some(operation), Some(placeholders)) = (
            conformance.operations.get(&usage.op_id),
            usage.placeholders.as_ref(),
        ) else {
            continue;
        };
        let expected = Conformance::path_placeholders(&operation.path);
        if &expected != placeholders {
            wrong.push(format!(
                "{} in {}: code has {placeholders:?}, spec path {} needs {expected:?}",
                usage.op_id, usage.file, operation.path
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn query_keys_exist_in_the_spec() {
    let conformance = Conformance::load();
    let mut wrong = Vec::new();
    for usage in &conformance.usages {
        let (Some(operation), Some(keys)) = (
            conformance.operations.get(&usage.op_id),
            usage.query_keys.as_ref(),
        ) else {
            continue;
        };
        for key in keys.difference(&operation.query_params) {
            wrong.push(format!(
                "{} in {}: query key `{key}` is not a parameter in the spec",
                usage.op_id, usage.file
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// `(struct, field)` pairs that are not in the response schema of every operation returning
/// the struct, on purpose: `Contact` is shared by character, corporation and alliance
/// contacts, and those schemas differ.
const FIELD_ALLOW_LIST: &[(&str, &str)] = &[("Contact", "is_blocked"), ("Contact", "is_watched")];

#[test]
fn struct_fields_match_the_response_schemas() {
    let conformance = Conformance::load();
    let mut wrong = Vec::new();
    let mut compared = 0;
    for usage in &conformance.usages {
        let (Some(operation), Some(type_name)) = (
            conformance.operations.get(&usage.op_id),
            usage.return_type.as_ref(),
        ) else {
            continue;
        };
        if let Some(item) = &usage.page_item {
            // `Page<T>` stands for the cursor page; compare `T` with a record.
            let (Some(spec_fields), Some(rust)) = (
                operation.page_item_fields.as_ref(),
                conformance.structs.get(item),
            ) else {
                wrong.push(format!(
                    "{} in {}: `Page<{item}>` does not match a cursor page of the spec",
                    usage.op_id, usage.file
                ));
                continue;
            };
            compared += 1;
            for field in rust.fields.iter().filter(|f| {
                !spec_fields.contains_key(*f)
                    && !FIELD_ALLOW_LIST.contains(&(item.as_str(), f.as_str()))
            }) {
                wrong.push(format!(
                    "{} in {}: `{item}.{field}` is not in the spec",
                    usage.op_id, usage.file
                ));
            }
            for (field, _) in spec_fields
                .iter()
                .filter(|(f, required)| **required && !rust.fields.contains(*f))
            {
                wrong.push(format!(
                    "{} in {}: required field `{field}` is missing from `{item}`",
                    usage.op_id, usage.file
                ));
            }
            continue;
        }
        let (Some(spec_fields), Some(rust)) = (
            operation.response_fields.as_ref(),
            conformance.structs.get(type_name),
        ) else {
            continue;
        };
        compared += 1;
        for field in rust.fields.iter().filter(|f| {
            !spec_fields.contains_key(*f)
                && !FIELD_ALLOW_LIST.contains(&(type_name.as_str(), f.as_str()))
        }) {
            wrong.push(format!(
                "{} in {}: `{type_name}.{field}` is not in the spec",
                usage.op_id, usage.file
            ));
        }
        for (field, _) in spec_fields
            .iter()
            .filter(|(f, required)| **required && !rust.fields.contains(*f))
        {
            wrong.push(format!(
                "{} in {}: required field `{field}` is missing from `{type_name}`",
                usage.op_id, usage.file
            ));
        }
    }
    assert!(compared > 100, "only {compared} responses were compared");
    wrong.sort();
    wrong.dedup();
    assert!(
        wrong.is_empty(),
        "{} differences:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// The scopes listed in `ESI_SCOPES` of `.env.example`, if the file exists.
struct ExampleScopes;

impl ExampleScopes {
    fn read() -> Option<BTreeSet<String>> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env.example");
        let text = fs::read_to_string(path).ok()?;
        let line = text.lines().find(|l| l.starts_with("ESI_SCOPES="))?;
        let value = line.trim_start_matches("ESI_SCOPES=").trim_matches('"');
        Some(value.split_whitespace().map(str::to_owned).collect())
    }

    fn spec_scopes() -> BTreeSet<String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let spec: Value = serde_json::from_str(
            &fs::read_to_string(root.join("resources/test/openapi.json")).unwrap(),
        )
        .unwrap();
        spec["components"]["securitySchemes"]["OAuth2"]["flows"]["authorizationCode"]["scopes"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

#[test]
fn example_scopes_exist_in_the_spec() {
    let Some(example) = ExampleScopes::read() else {
        eprintln!("skipped: .env.example has no ESI_SCOPES");
        return;
    };
    let known = ExampleScopes::spec_scopes();
    let unknown: Vec<&String> = example.difference(&known).collect();
    assert!(unknown.is_empty(), "scopes not in the spec: {unknown:?}");
}

#[test]
fn example_scopes_cover_every_scope_the_operations_need() {
    let Some(example) = ExampleScopes::read() else {
        eprintln!("skipped: .env.example has no ESI_SCOPES");
        return;
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spec: Value = serde_json::from_str(
        &fs::read_to_string(root.join("resources/test/openapi.json")).unwrap(),
    )
    .unwrap();
    let mut needed = BTreeSet::new();
    for item in spec["paths"].as_object().unwrap().values() {
        for method in HTTP_METHODS {
            let scopes = item[method]["security"][0]["OAuth2"].as_array();
            needed.extend(
                scopes
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned),
            );
        }
    }
    let missing: Vec<&String> = needed.difference(&example).collect();
    assert!(
        missing.is_empty(),
        "scopes missing from .env.example: {missing:?}"
    );
}

#[test]
fn chunk_sizes_equal_the_spec_maximums() {
    // `Chunked(body: &[T], max)` calls must use the `maxItems` of the request body.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spec: Value = serde_json::from_str(
        &fs::read_to_string(root.join("resources/test/openapi.json")).unwrap(),
    )
    .unwrap();
    let mut limits = BTreeMap::new();
    for item in spec["paths"].as_object().unwrap().values() {
        let Some(operation) = item.get("post") else {
            continue;
        };
        let schema = &operation["requestBody"]["content"]["application/json"]["schema"];
        let schema = Conformance::resolve(&spec, schema);
        if let (Some(id), Some(max)) = (
            operation["operationId"].as_str(),
            schema.get("maxItems").and_then(Value::as_u64),
        ) {
            limits.insert(id.to_owned(), max);
        }
    }
    let mut checked = 0;
    for entry in fs::read_dir(root.join("src/groups")).unwrap() {
        let source = fs::read_to_string(entry.unwrap().path()).unwrap();
        for call in source.split("api_post!(").skip(1) {
            let call = &call[..call.find("\n    );").unwrap_or(call.len())];
            let Some(start) = call.find("Chunked(") else {
                continue;
            };
            let op_id = Conformance::literals(call)[0];
            let max: u64 = call[start..]
                .rsplit(',')
                .next()
                .unwrap()
                .trim()
                .trim_end_matches(')')
                .trim()
                .parse()
                .unwrap();
            assert_eq!(limits.get(op_id), Some(&max), "{op_id}");
            checked += 1;
        }
    }
    assert_eq!(checked, 6, "expected six chunked endpoints");
}

#[test]
fn cursor_pages_use_keys_that_page_accepts() {
    // `Page::items` reads the records under these keys (see `serde(alias)`).
    const KEYS: [&str; 6] = [
        "projects",
        "contributors",
        "freelance_jobs",
        "participants",
        "objectives",
        "listings",
    ];
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spec: Value = serde_json::from_str(
        &fs::read_to_string(root.join("resources/test/openapi.json")).unwrap(),
    )
    .unwrap();
    let mut checked = 0;
    for item in spec["paths"].as_object().unwrap().values() {
        let Some(operation) = item.get("get") else {
            continue;
        };
        if operation["x-pagination"] != "cursor" {
            continue;
        }
        let ok = operation["responses"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(code, _)| code.starts_with('2'))
            .unwrap()
            .1;
        let ok = Conformance::resolve(&spec, ok);
        let schema = &ok["content"]["application/json"]["schema"];
        let schema = Conformance::resolve(&spec, schema);
        let key = schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .find(|k| *k != "cursor")
            .unwrap();
        assert!(
            KEYS.contains(&key.as_str()),
            "{}: `{key}` is not a key of `Page`",
            operation["operationId"]
        );
        checked += 1;
    }
    assert!(checked >= 12, "only {checked} cursor operations were found");
}
