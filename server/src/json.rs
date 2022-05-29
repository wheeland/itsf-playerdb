#[derive(serde::Serialize)]
struct JsonOk<T: serde::Serialize> {
    data: T,
}

#[derive(serde::Serialize)]
struct JsonErr<T: serde::Serialize> {
    error: T,
}

pub fn ok<T: serde::Serialize>(data: T) -> String {
    let ok = JsonOk { data };
    serde_json::to_string(&ok).unwrap()
}

pub fn err<T: serde::Serialize>(error: T) -> String {
    let err = JsonErr { error };
    serde_json::to_string(&err).unwrap()
}
