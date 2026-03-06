use std::path::PathBuf;

use egui::{Id, ScrollArea};
use egui_file_dialog::FileDialog;
use egui_ltreeview::{NodeBuilder, TreeView, TreeViewBuilder, TreeViewState};

use crate::florr::data::{Dir, File, Node};
use crate::florr::extract::{Asset, compress_to_zip, extract_wasm};

pub struct Florr {
    file_dialog: FileDialog,
    file_path: Option<PathBuf>,
    tree: Node,
    tree_view_state: TreeViewState<String>,
    assets: Vec<Asset>
}

impl Florr {
    pub fn new() -> Self {
        Self {
            file_dialog: FileDialog::new().default_file_filter("wasm"),
            file_path: None,
            tree: Node::dir("Root", Vec::new()),
            tree_view_state: TreeViewState::default(),
            assets: Vec::new()
        }
    }
}

impl eframe::App for Florr {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left(Id::new("tree"))
            .resizable(true)
            .show(ctx, |ui| {
                ScrollArea::both().show(ui, |ui| {

                    if !self.assets.is_empty() && ui.button("Download as .zip").clicked() {
                        let _ = compress_to_zip(&self.assets);
                    }

                    let view = TreeView::new(Id::new("tree view"));
                    view.show_state(ui, &mut self.tree_view_state, |builder| {
                        show_node(&self.tree, builder);
                        builder.close_dir();
                    })
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FlorrView");

            ui.horizontal(|ui| {
                if ui.button("Insert client.wasm file here").clicked() {
                    self.file_dialog.pick_file();
                }

                let path_name;
                match &self.file_path {
                    Some(path) => {
                        let strip: PathBuf = path.components().skip(4).collect();
                        path_name = Some(strip);
                    }
                    None => {
                        path_name = None;
                    }
                }
                ui.label(format!("- {:?}", path_name));
            });

            self.file_dialog.update(ctx);

            ui.separator();

            egui::ScrollArea::both().show(ui, |ui| {
                if let Node::File(_) = &self.tree {
                    ui.label("Tree Error");
                    return;
                };
                if let Some(path) = self.file_dialog.take_picked() {
                    if let Node::Dir(dir) = &mut self.tree {
                        dir.children.clear();
                    }
                    match extract_wasm(&path) {
                        Ok(assets) => {
                            for asset in &assets {
                                let path = &asset.path;
                                let content = &asset.content;

                                let mut current_node = &mut self.tree;
                                let mut current_path = String::new();
                                // let mut prev;

                                let s: Vec<&str> = path.split('/').collect();
                                for i in 0..s.len() - 1 {
                                    let name = s[i];
                                    if !current_path.is_empty() {
                                        current_path.push('/');
                                    }
                                    current_path.push_str(name);

                                    if let Node::Dir(dir) = current_node {
                                        current_node = dir.get_or_create_dir(&current_path);
                                    }
                                }

                                if let Node::Dir(dir) = current_node {
                                    dir.children.push(Node::file(path, content));
                                }
                            }
                            self.assets = assets;
                        }
                        Err(err) => {
                            eprintln!("{err}");
                        }
                    }
                    self.file_path = Some(path);
                }

                if let Some(id) = self.tree_view_state.selected().last() {
                    if let Some(node) = self.tree.find_mut(id) {
                        let Node::File(file) = node else {
                            return;
                        };
                        ui.label(&file.content);
                    }
                }
            });
        });
    }
}

// a

fn show_node(node: &Node, builder: &mut TreeViewBuilder<String>) {
    match node {
        Node::Dir(dir) => show_dir(dir, builder),
        Node::File(file) => show_file(file, builder),
    }
}

fn show_dir(dir: &Dir, builder: &mut TreeViewBuilder<String>) {
    let node = NodeBuilder::dir(dir.id.clone()).label(&dir.name);
    builder.node(node);
    for node in dir.children.iter() {
        show_node(node, builder);
    }
    builder.close_dir();
}

fn show_file(file: &File, builder: &mut TreeViewBuilder<String>) {
    let node = NodeBuilder::leaf(file.id.clone()).label(&file.name);
    builder.node(node);
}
