use super::{child_buffer::ChildBuffer, internal::TakeChildBuffer, Inner, Node, Tree};
use crate::{
    cache::AddSize,
    data_management::{HandlerDml, HasStoragePreference, ObjectRef},
    size::Size,
    tree::{errors::*, MessageAction},
};
use std::borrow::Borrow;

impl<X, R, M, I> Tree<X, M, I>
where
    X: HandlerDml<Object = Node<R>, ObjectRef = R>,
    R: ObjectRef<ObjectPointer = X::ObjectPointer> + HasStoragePreference,
    M: MessageAction,
    I: Borrow<Inner<X::ObjectRef, X::Info, M>>,
{
    pub(super) fn split_root_node(&self, mut root_node: X::CacheValueRefMut) {
        self.dml.verify_cache();
        let before = root_node.size();
        info!(
            "Splitting root. {}, {:?}, {}, {:?}",
            root_node.kind(),
            root_node.fanout(),
            root_node.size(),
            root_node.actual_size()
        );
        let size_delta = root_node.split_root_mut(|node| {
            info!(
                "Root split child: {}, {:?}, {}, {:?}",
                node.kind(),
                node.fanout(),
                node.size(),
                node.actual_size()
            );
            self.dml.insert(node, self.tree_id())
        });
        info!("Root split done. {}, {}", root_node.size(), size_delta);
        assert!(before as isize + size_delta == root_node.size() as isize);
        root_node.finish(size_delta);
        self.dml.verify_cache();
    }

    pub(super) fn split_node(
        &self,
        mut node: X::CacheValueRefMut,
        parent: &mut TakeChildBuffer<ChildBuffer<R>>,
    ) -> Result<(X::CacheValueRefMut, isize), Error> {
        self.dml.verify_cache();

        let before = node.size();
        let (sibling, pivot_key, size_delta) = node.split();
        let select_right = sibling.size() > node.size();
        info!(
            "split {}: {} -> ({}, {}), {}",
            node.kind(),
            before,
            node.size(),
            sibling.size(),
            select_right,
        );
        node.add_size(size_delta);
        let sibling_np = if select_right {
            let (sibling, np) = self.dml.insert_and_get_mut(sibling, self.tree_id());
            node = sibling;
            np
        } else {
            self.dml.insert(sibling, self.tree_id())
        };

        let size_delta = parent.split_child(sibling_np, pivot_key, select_right);

        Ok((node, size_delta))
    }
}
