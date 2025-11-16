use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaDocs {
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub package: String,
    pub description: String,
    pub classes: Vec<Class>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub overloads: Vec<Overload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overload {
    pub signature: String,
    pub description: String,
    #[serde(default)]
    pub deprecated: bool,
}

/// Index for fast lookups
#[derive(Debug)]
pub struct DocsIndex {
    /// Map from fully qualified class name to (package_idx, class_idx)
    pub classes: HashMap<String, (usize, usize)>,
    /// Map from method name to list of (package_idx, class_idx, method_idx)
    pub methods: HashMap<String, Vec<(usize, usize, usize)>>,
    /// The actual docs data
    pub docs: JavaDocs,
}

impl DocsIndex {
    /// Build an index from the parsed docs
    pub fn build(docs: JavaDocs) -> Self {
        let mut classes = HashMap::new();
        let mut methods: HashMap<String, Vec<(usize, usize, usize)>> = HashMap::new();

        for (pkg_idx, package) in docs.packages.iter().enumerate() {
            for (cls_idx, class) in package.classes.iter().enumerate() {
                // Index fully qualified class name
                let fqn = format!("{}.{}", package.package, class.name);
                classes.insert(fqn.clone(), (pkg_idx, cls_idx));

                // Also index just the class name
                classes.insert(class.name.clone(), (pkg_idx, cls_idx));

                // Index methods
                for (method_idx, method) in class.methods.iter().enumerate() {
                    methods
                        .entry(method.name.clone())
                        .or_insert_with(Vec::new)
                        .push((pkg_idx, cls_idx, method_idx));
                }
            }
        }

        DocsIndex {
            classes,
            methods,
            docs,
        }
    }

    /// Lookup a class by name (supports both simple and fully qualified names)
    pub fn get_class(&self, name: &str) -> Option<&Class> {
        self.classes
            .get(name)
            .map(|(pkg_idx, cls_idx)| &self.docs.packages[*pkg_idx].classes[*cls_idx])
    }

    /// Lookup a class and its package by name
    pub fn get_class_with_package(&self, name: &str) -> Option<(&Package, &Class)> {
        self.classes.get(name).map(|(pkg_idx, cls_idx)| {
            let package = &self.docs.packages[*pkg_idx];
            let class = &package.classes[*cls_idx];
            (package, class)
        })
    }

    /// Lookup all methods with a given name across all classes
    pub fn get_methods(&self, name: &str) -> Vec<(&Package, &Class, &Method)> {
        self.methods
            .get(name)
            .map(|indices| {
                indices
                    .iter()
                    .map(|(pkg_idx, cls_idx, method_idx)| {
                        let package = &self.docs.packages[*pkg_idx];
                        let class = &package.classes[*cls_idx];
                        let method = &class.methods[*method_idx];
                        (package, class, method)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Lookup a specific method in a specific class
    pub fn get_class_method(&self, class_name: &str, method_name: &str) -> Option<&Method> {
        self.classes.get(class_name).and_then(|(pkg_idx, cls_idx)| {
            self.docs.packages[*pkg_idx].classes[*cls_idx]
                .methods
                .iter()
                .find(|m| m.name == method_name)
        })
    }

    /// Get all classes in a package
    pub fn get_package(&self, package_name: &str) -> Option<&Package> {
        self.docs
            .packages
            .iter()
            .find(|p| p.package == package_name)
    }

    /// Search for classes by partial name match
    pub fn search_classes(&self, query: &str) -> Vec<(String, &Class)> {
        let query_lower = query.to_lowercase();
        self.classes
            .iter()
            .filter(|(name, _)| name.to_lowercase().contains(&query_lower))
            .map(|(name, (pkg_idx, cls_idx))| {
                (
                    name.clone(),
                    &self.docs.packages[*pkg_idx].classes[*cls_idx],
                )
            })
            .collect()
    }
}
