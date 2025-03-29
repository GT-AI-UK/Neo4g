pub fn static_string_bench() {
    let string = "MERGE (g:Group:Any { id: $group_id })
    ON CREATE SET 
        g.name = $name,
        g.created = datetime($now),
        g.updated = datetime($now),
        g.deleted = false
    ON MATCH SET
        g.deleted = false

    WITH g, $member_ids AS members, $member_ofs AS groups

    CALL {
        WITH g, members
        WITH g, members WHERE size(members) > 0
        UNWIND members AS member

        OPTIONAL MATCH (gm {id: member})
        WITH g, gm WHERE gm IS NOT NULL

        MERGE (gm)-[mo:MEMBER_OF]->(g)
            ON CREATE SET
                mo.created = datetime($now),
                mo.updated = datetime($now),
                mo.deleted = false

        WITH g, COLLECT(gm) AS expected_members
        OPTIONAL MATCH (gm)-[mo:MEMBER_OF]->(g)
        WHERE NOT gm IN expected_members
        SET mo.updated = datetime($now), mo.deleted = true
    }

    CALL {
        WITH g, groups
        WITH g, groups WHERE size(groups) > 0
        UNWIND groups AS group_name

        OPTIONAL MATCH (o {id: group_name})
        WITH g, o WHERE o IS NOT NULL

        MERGE (g)-[mo:MEMBER_OF]->(o)
            ON CREATE SET
                mo.created = datetime($now),
                mo.updated = datetime($now),
                mo.deleted = false

        WITH g, COLLECT(o) AS expected_groups
        OPTIONAL MATCH (g)-[mo:MEMBER_OF]->(o)
        WHERE NOT o IN expected_groups
        DELETE mo
    }

    MATCH (g:Group:Any { id: $group_id })
    RETURN g";
}