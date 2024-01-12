from typed_graph import GenericGraph, GenericSchema, GenericWeight

if __name__ == '__main__':
    s = GenericSchema[int, int, int, int](
        node_blacklist = [3],
        edge_blacklist = [3],
        endpoint_max_quantity = {
            (0, 1, 0): 2
        }
    )

    g = GenericGraph[int, int, int, int](s)

    g.add_node(GenericWeight((0, 0)))
    g.add_node(GenericWeight((1, 1)))

    try:
        g.add_node(GenericWeight((2, 3)))
    except Exception as e:
        print('Invalid: ', e)

    g.add_edge(0, 1, GenericWeight((0, 0)))
    g.add_edge(0, 1, GenericWeight((1, 0)))

    try:
        g.add_edge(0, 1, GenericWeight((2, 3)))
    except Exception as e:
        print('Invalid: ', e)

    try:
        g.add_edge(0, 1, GenericWeight((2, 0)))
    except Exception as e:
        print('ToMany: ', e)

    n = g.get_node(0)

    print('Id =', n[0], 'or', n.get_id(), ' Type =', n[1], 'or', n.get_type())