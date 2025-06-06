// CRUD operations for benchmarking
QUERY create_record(data: String) =>
    record <- AddN<Record>({ data: data })
    RETURN record

QUERY read_record(id: ID) =>
    record <- N<Record>(id)
    RETURN record

QUERY update_record(id: ID, data: String) =>
    record <- N<Record>(id)::UPDATE({
        data: data
    })
    RETURN record

QUERY delete_record(id: ID) =>
    DROP N<Record>(id)
    RETURN "NONE"

QUERY scan_records(limit: I32, offset: I32) =>
    records <- N<Record>::RANGE(offset, limit)
    RETURN records

QUERY count_records() =>
    count <- N<Record>::COUNT
    RETURN count

QUERY create_vector(vec: [F64]) =>
    AddV<Embedding>(vec)
    RETURN "SUCCESS"

QUERY search_vector(query: [F64], k: I32) =>
    vec <- SearchV<Embedding>(query, k)
    RETURN vec

//QUERY bulk_add(data: [String]) =>
//    FOR d IN data {
//        AddN<Record>({ data: d })
//    }
//    RETURN "SUCCESS"
