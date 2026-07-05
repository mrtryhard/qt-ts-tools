#[derive(Debug, PartialEq)]
pub struct DependenciesNode {
    pub dependencies: Vec<DependencyNode>,
}

#[derive(Debug, Default, PartialEq)]
pub struct DependencyNode {
    pub catalog: String,
}