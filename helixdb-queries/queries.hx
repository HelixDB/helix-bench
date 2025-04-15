// CRUD operations for benchmarking
QUERY create_record(id: String, data: String) =>
    record <- AddN<Record>({
        id: id,
        data: data
    })
    RETURN record



QUERY read_record(id: String) =>
    record <- N<Record>(id)
    RETURN record

QUERY update_record(id: String, data: String) =>
    record <- N<Record>(id)::UPDATE({
        data: data
    })
    RETURN record

QUERY delete_record(id: String) =>
    DROP N<Record>(id)
    RETURN NONE

QUERY scan_records(limit: Integer, offset: Integer) =>
    records <- N<Record>::RANGE(offset, limit)
    RETURN records

QUERY count_records() =>
    count <- N<Record>::COUNT
    RETURN count

// QUERY CreateUserWithRole(roleType: String) =>
//     newUser <- AddN<User>
//     role <- N<Role>::WHERE(_::{role_type}::EQ("userRole"))
//     roleRelationship <- AddE<HasRole>::From(newUser)::To(role)
//     RETURN newUser
// 
// QUERY CreateRoleType() =>
//     role <- AddN<RoleType>({
//         role_type: "userRole"
//     })
//     RETURN role