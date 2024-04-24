use itertools::{repeat_n, Itertools};

#[derive(Debug)]
pub struct ProxyState {
    pub routes: Vec<NodePath>,
}

impl ProxyState {
    pub fn new() -> ProxyState {
        ProxyState { routes: vec![] }
    }
    pub fn append_path(&mut self, path: &Vec<String>) {
        let mut root = &mut self.routes;

        for p in path.iter() {
            let next_node_idx = ProxyState::get_child_idx(&root, p);

            root = match next_node_idx {
                Some(x) => &mut root[x].children,
                None => {
                    let new_node = NodePath::constant(p.clone());
                    root.push(new_node);
                    &mut root.last_mut().unwrap().children
                }
            }
        }
    }

    fn get_child_idx(v: &Vec<NodePath>, curr_c: &String) -> Option<usize> {
        v.iter()
            .enumerate()
            .find(|(_, n)| n.value.eq(curr_c))
            .map(|(i, _)| i)
    }
}

#[derive(Debug)]
pub struct NodePath {
    pub value: String,
    pub children: Vec<NodePath>,
}

impl NodePath {
    pub fn constant(str: String) -> NodePath {
        NodePath {
            value: str,
            children: vec![],
        }
    }

    pub fn display(&self, offset: usize) -> String {
        format!(
            "{}{}\n{}",
            repeat_n(" -> ", offset).into_iter().format(""),
            &self.value,
            self.children
                .iter()
                .map(|c| c.display(offset + 1))
                .format("")
        )
    }
}
