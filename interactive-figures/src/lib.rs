#[derive(serde::Deserialize, serde::Serialize)]
pub struct Figure {
    pub paths: Vec<pxu::Path>,
    pub state: pxu::State,
    pub consts: pxu::CouplingConstants,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FigureDescription {
    pub name: String,
    pub description: String,
    pub filename: String,
    pub consts: pxu::CouplingConstants,
    pub paper_ref: Vec<String>,
}
