
// https://github.com/LennysLounge/egui_ltreeview
// THANKS VERY MUCH helpful examples

pub enum Node {
    Dir(Dir),
    File(File),
}
pub struct Dir {
    pub id: String,
    pub name: String,
    pub children: Vec<Node>,
}
pub struct File {
    pub id: String,
    pub name: String,
    pub content: String,
}

impl Node {
    pub fn dir(id: &str, children: Vec<Node>) -> Self {
        let name = id.split('/').last().unwrap_or(id);
        Node::Dir(Dir {
            id: id.to_string(),
            name: name.to_string(),
            children,
        })
    }

    pub fn file(path: &str, content: &str) -> Self {
        let name = path.split('/').last().unwrap_or(path);
        Node::File(File {
            id: path.to_string(),
            name: name.to_string(),
            content: content.to_string(),
        })
    }

    pub fn id(&self) -> &str {
        match self {
            Node::Dir(dir) => &dir.id,
            Node::File(file) => &file.id,
        }
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut Node> {
        if self.id() == id {
            return Some(self);
        } else {
            match self {
                Node::Dir(dir) => {
                    for node in dir.children.iter_mut() {
                        if let Some(node) = node.find_mut(id) {
                            return Some(node);
                        }
                    }
                }
                Node::File(_) => (),
            }
        }
        None
    }
}

impl Dir {
    pub fn get_or_create_dir(&mut self, id: &str) -> &mut Node {
        let pos = self.children.iter().position(|child| child.id() == id);

        if let Some(index) = pos {
            &mut self.children[index]
        } else {
            self.children.push(Node::dir(id, Vec::new()));
            self.children.last_mut().unwrap()
        }
    }
}
