module rooch_fish::quad_tree {
    use std::vector;
    use moveos_std::table::{Self, Table};

    const MAX_DEPTH: u8 = 5;
    const MAX_OBJECTS: u64 = 2;

    struct Point has copy, drop, store {
        x: u64,
        y: u64,
    }

    struct Rectangle has copy, drop, store {
        x: u64,
        y: u64,
        width: u64,
        height: u64,
    }

    struct ObjectEntry<T: copy + drop + store> has copy, drop, store {
        id: T,
        object_type: u8,
        x: u64,
        y: u64,
    }

    struct QuadTreeNode<T: copy + drop + store> has copy, drop, store {
        boundary: Rectangle,
        objects: vector<ObjectEntry<T>>,
        is_divided: bool,
        nw: u64,
        ne: u64,
        sw: u64,
        se: u64,
    }

    struct QuadTree<T: copy + drop + store> has key, store {
        nodes: Table<u64, QuadTreeNode<T>>,
        next_node_id: u64,
        root: u64,
        width: u64,
        height: u64,
    }

    public fun create_quad_tree<T: copy + drop + store>(width: u64, height: u64): QuadTree<T> {
        let root_node = QuadTreeNode<T> {
            boundary: Rectangle { x: 0, y: 0, width, height },
            objects: vector::empty(),
            is_divided: false,
            nw: 0, ne: 0, sw: 0, se: 0,
        };
        let nodes = table::new();
        table::add(&mut nodes, 0, root_node);
        QuadTree<T> {
            nodes,
            next_node_id: 1,
            root: 0,
            width,
            height,
        }
    }

    public fun insert_object<T: copy + drop + store>(tree: &mut QuadTree<T>, id: T, object_type: u8, x: u64, y: u64) {
        insert_object_recursive(tree, ObjectEntry { id, object_type, x, y }, 0, 0);
    }

    fun insert_object_recursive<T: copy + drop + store>(
        tree: &mut QuadTree<T>,
        object: ObjectEntry<T>,
        node_id: u64,
        depth: u8
    ) {
        if (depth >= MAX_DEPTH) {
            return
        };

        let node = table::borrow_mut(&mut tree.nodes, node_id);
        if (!node.is_divided) {
            if (vector::length(&node.objects) < MAX_OBJECTS) {
                vector::push_back(&mut node.objects, object);
                return
            };
            subdivide(tree, node_id);
        };

        let node = table::borrow(&tree.nodes, node_id);
        if (object.x < node.boundary.x + node.boundary.width / 2) {
            if (object.y < node.boundary.y + node.boundary.height / 2) {
                insert_object_recursive(tree, object, node.nw, depth + 1);
            } else {
                insert_object_recursive(tree, object, node.sw, depth + 1);
            };
        } else {
            if (object.y < node.boundary.y + node.boundary.height / 2) {
                insert_object_recursive(tree, object, node.ne, depth + 1);
            } else {
                insert_object_recursive(tree, object, node.se, depth + 1);
            };
        };
    }

    fun subdivide<T: copy + drop + store>(tree: &mut QuadTree<T>, node_id: u64) {
        let node = table::borrow_mut(&mut tree.nodes, node_id);
        if (node.is_divided) {
            return
        };

        let x = node.boundary.x;
        let y = node.boundary.y;
        let w = node.boundary.width / 2;
        let h = node.boundary.height / 2;

        let nw = tree.next_node_id;
        tree.next_node_id = tree.next_node_id + 1;
        let ne = tree.next_node_id;
        tree.next_node_id = tree.next_node_id + 1;
        let sw = tree.next_node_id;
        tree.next_node_id = tree.next_node_id + 1;
        let se = tree.next_node_id;
        tree.next_node_id = tree.next_node_id + 1;

        table::add(&mut tree.nodes, nw, QuadTreeNode<T> {
            boundary: Rectangle { x, y, width: w, height: h },
            objects: vector::empty(),
            is_divided: false,
            nw: 0, ne: 0, sw: 0, se: 0,
        });
        table::add(&mut tree.nodes, ne, QuadTreeNode<T> {
            boundary: Rectangle { x: x + w, y, width: w, height: h },
            objects: vector::empty(),
            is_divided: false,
            nw: 0, ne: 0, sw: 0, se: 0,
        });
        table::add(&mut tree.nodes, sw, QuadTreeNode<T> {
            boundary: Rectangle { x, y: y + h, width: w, height: h },
            objects: vector::empty(),
            is_divided: false,
            nw: 0, ne: 0, sw: 0, se: 0,
        });
        table::add(&mut tree.nodes, se, QuadTreeNode<T> {
            boundary: Rectangle { x: x + w, y: y + h, width: w, height: h },
            objects: vector::empty(),
            is_divided: false,
            nw: 0, ne: 0, sw: 0, se: 0,
        });

        let node = table::borrow_mut(&mut tree.nodes, node_id);
        node.nw = nw;
        node.ne = ne;
        node.sw = sw;
        node.se = se;
        node.is_divided = true;
    }

    public fun query_range<T: copy + drop + store>(
        tree: &QuadTree<T>,
        x: u64,
        y: u64,
        width: u64,
        height: u64
    ): vector<ObjectEntry<T>> {
        let range = Rectangle { x, y, width, height };
        let result = vector::empty();
        query_range_recursive(tree, tree.root, &range, &mut result);
        result
    }

    fun query_range_recursive<T: copy + drop + store>(
        tree: &QuadTree<T>,
        node_id: u64,
        range: &Rectangle,
        result: &mut vector<ObjectEntry<T>>
    ) {
        let node = table::borrow(&tree.nodes, node_id);
        if (!intersects(&node.boundary, range)) {
            return
        };

        let i = 0;
        while (i < vector::length(&node.objects)) {
            let object_entry = vector::borrow(&node.objects, i);
            if (is_point_in_rectangle(object_entry.x, object_entry.y, range)) {
                vector::push_back(result, *object_entry);
            };
            i = i + 1;
        };

        if (node.is_divided) {
            query_range_recursive(tree, node.nw, range, result);
            query_range_recursive(tree, node.ne, range, result);
            query_range_recursive(tree, node.sw, range, result);
            query_range_recursive(tree, node.se, range, result);
        };
    }

    fun intersects(r1: &Rectangle, r2: &Rectangle): bool {
        !(r2.x > r1.x + r1.width ||
          r2.x + r2.width < r1.x ||
          r2.y > r1.y + r1.height ||
          r2.y + r2.height < r1.y)
    }

    fun is_point_in_rectangle(x: u64, y: u64, rect: &Rectangle): bool {
        x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
    }

    public fun remove_object<T: copy + drop + store>(tree: &mut QuadTree<T>, id: T, object_type: u8, x: u64, y: u64) {
        remove_object_recursive(tree, id, object_type, x, y, 0);
    }

    fun remove_object_recursive<T: copy + drop + store>(
        tree: &mut QuadTree<T>,
        id: T,
        object_type: u8,
        x: u64,
        y: u64,
        node_id: u64
    ) {
        let node = table::borrow(&tree.nodes, node_id);
        // First check if the point is within current node's boundary
        if (!is_point_in_rectangle(x, y, &node.boundary)) {
            return
        };

        // Check if we need to process current node
        if (!node.is_divided) {
            let node = table::borrow_mut(&mut tree.nodes, node_id);
            let i = 0;
            while (i < vector::length(&node.objects)) {
                let object_entry = vector::borrow(&node.objects, i);
                if (object_entry.id == id && object_entry.object_type == object_type && object_entry.x == x && object_entry.y == y) {
                    vector::remove(&mut node.objects, i);
                    return
                };
                i = i + 1;
            };
            return
        };

        // If node is divided, get quadrant information first
        let mid_x = node.boundary.x + node.boundary.width / 2;
        let mid_y = node.boundary.y + node.boundary.height / 2;
        let nw = node.nw;
        let ne = node.ne;
        let sw = node.sw;
        let se = node.se;
 
        // Search appropriate quadrant
        if (x < mid_x) {
            if (y < mid_y) {
                remove_object_recursive(tree, id, object_type, x, y, nw);
            } else {
                remove_object_recursive(tree, id, object_type, x, y, sw);
            };
        } else {
            if (y < mid_y) {
                remove_object_recursive(tree, id, object_type, x, y, ne);
            } else {
                remove_object_recursive(tree, id, object_type, x, y, se);
            };
        };
    }


    public fun update_object_position<T: copy + drop + store>(
        tree: &mut QuadTree<T>,
        id: T,
        object_type: u8,
        old_x: u64,
        old_y: u64,
        new_x: u64,
        new_y: u64
    ) {
        remove_object(tree, id, object_type, old_x, old_y);
        insert_object(tree, id, object_type, new_x, new_y);
    }

    public fun drop_quad_tree<T: copy + drop + store>(tree: QuadTree<T>) {
        let QuadTree { nodes, next_node_id: _, root:_, width: _, height: _ } = tree;
        table::drop(nodes);
    }

    public fun get_object_entry_id<T: copy + drop + store>(entry: &ObjectEntry<T>): T {
        entry.id
    }

    public fun get_object_entry_type<T: copy + drop + store>(entry: &ObjectEntry<T>): u8 {
        entry.object_type
    }

    public fun get_object_entry_x<T: copy + drop + store>(entry: &ObjectEntry<T>): u64 {
        entry.x
    }

    public fun get_object_entry_y<T: copy + drop + store>(entry: &ObjectEntry<T>): u64 {
        entry.y
    }

    #[test]
    fun test_create_quad_tree() {
        let tree = create_quad_tree<u64>(100, 100);
        drop_quad_tree(tree);
    }

    #[test]
    fun test_insert_and_query() {
        let tree = create_quad_tree<u64>(100, 100);
        
        insert_object(&mut tree, 1, 1, 10, 10);
        insert_object(&mut tree, 2, 1, 20, 20);
        insert_object(&mut tree, 3, 1, 30, 30);

        let result = query_range(&tree, 0, 0, 50, 50);
        assert!(vector::length(&result) == 3, 0);

        let result = query_range(&tree, 0, 0, 15, 15);
        assert!(vector::length(&result) == 1, 1);

        let result = query_range(&tree, 5, 5, 10, 10);
        assert!(vector::length(&result) == 1, 2);

        let result = query_range(&tree, 15, 15, 10, 10);
        assert!(vector::length(&result) == 1, 3);

        let result = query_range(&tree, 25, 25, 10, 10);
        assert!(vector::length(&result) == 1, 4);

        let result = query_range(&tree, 40, 40, 10, 10);
        assert!(vector::length(&result) == 0, 5);

        drop_quad_tree(tree);
    }

    #[test]
    fun test_remove_object() {
        let tree = create_quad_tree<u64>(100, 100);
        
        insert_object(&mut tree, 1, 1, 10, 10);
        insert_object(&mut tree, 2, 1, 20, 20);

        let result = query_range(&tree, 0, 0, 50, 50);
        assert!(vector::length(&result) == 2, 0);

        remove_object(&mut tree, 1, 1, 10, 10);

        let result = query_range(&tree, 0, 0, 50, 50);
        assert!(vector::length(&result) == 1, 1);

        drop_quad_tree(tree);
    }

    #[test]
    fun test_update_object_position() {
        let tree = create_quad_tree<u64>(100, 100);
        
        insert_object(&mut tree, 1, 1, 10, 10);

        let result = query_range(&tree, 0, 0, 15, 15);
        assert!(vector::length(&result) == 1, 0);

        update_object_position(&mut tree, 1, 1, 10, 10, 50, 50);

        let result = query_range(&tree, 0, 0, 15, 15);
        assert!(vector::length(&result) == 0, 1);

        let result = query_range(&tree, 45, 45, 10, 10);
        assert!(vector::length(&result) == 1, 2);

        drop_quad_tree(tree);
    }

    #[test]
    fun test_get_object_entry_functions() {
        let tree = create_quad_tree<u64>(100, 100);
        
        insert_object(&mut tree, 1, 2, 10, 10);

        let result = query_range(&tree, 0, 0, 15, 15);
        assert!(vector::length(&result) == 1, 0);

        let entry = vector::borrow(&result, 0);
        assert!(get_object_entry_id(entry) == 1, 1);
        assert!(get_object_entry_type(entry) == 2, 2);
        assert!(get_object_entry_x(entry) == 10, 3);
        assert!(get_object_entry_y(entry) == 10, 4);

        drop_quad_tree(tree);
    }

    #[test]
    fun test_remove_object_at_boundaries() {
        let tree = create_quad_tree<u64>(100, 100);
        
        // Insert objects at quadrant boundaries
        insert_object(&mut tree, 1, 1, 50, 50);
        
        let result = query_range(&tree, 45, 45, 10, 10);
        assert!(vector::length(&result) == 1, 0);
        
        remove_object(&mut tree, 1, 1, 50, 50);
        
        let result = query_range(&tree, 45, 45, 10, 10);
        assert!(vector::length(&result) == 0, 1);

        drop_quad_tree(tree);
    }
}
