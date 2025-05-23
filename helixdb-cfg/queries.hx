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
