diesel::table! {
  use diesel::sql_types::*;
  // use pgvector::sql_types::*;

  events (id) {
      id -> Uuid,
      workflow_type -> Text,
      data -> Json,
      task_context -> Json,
      created_at -> Timestamptz,
      updated_at -> Timestamptz,
  }
}

diesel::table! {
    use diesel::sql_types::*;

    agents (id) {
        id -> Uuid,
        name -> Varchar,
        endpoint -> Varchar,
        capabilities -> Array<Text>,
        status -> Varchar,
        last_seen -> Timestamptz,
        metadata -> Json,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    event_store (id) {
        id -> Uuid,
        aggregate_id -> Uuid,
        aggregate_type -> Varchar,
        event_type -> Varchar,
        aggregate_version -> Int8,
        event_data -> Json,
        metadata -> Json,
        occurred_at -> Timestamptz,
        recorded_at -> Timestamptz,
        schema_version -> Int4,
        causation_id -> Nullable<Uuid>,
        correlation_id -> Nullable<Uuid>,
        checksum -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    event_snapshots (id) {
        id -> Uuid,
        aggregate_id -> Uuid,
        aggregate_type -> Varchar,
        aggregate_version -> Int8,
        snapshot_data -> Json,
        created_at -> Timestamptz,
        metadata -> Json,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    event_subscriptions (id) {
        id -> Uuid,
        subscription_name -> Varchar,
        event_types -> Array<Text>,
        last_processed_position -> Nullable<Int8>,
        status -> Varchar,
        filter_criteria -> Json,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    event_dead_letter_queue (id) {
        id -> Uuid,
        original_event_id -> Uuid,
        event_data -> Json,
        error_message -> Text,
        error_details -> Json,
        retry_count -> Nullable<Int4>,
        max_retries -> Nullable<Int4>,
        status -> Varchar,
        created_at -> Timestamptz,
        last_retry_at -> Nullable<Timestamptz>,
        next_retry_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    event_projections (id) {
        id -> Uuid,
        projection_name -> Varchar,
        last_processed_event_id -> Nullable<Uuid>,
        last_processed_position -> Nullable<Int8>,
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        salt -> Varchar,
        is_active -> Bool,
        role -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_login -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    tenants (id) {
        id -> Uuid,
        name -> Varchar,
        database_schema -> Varchar,
        isolation_mode -> Varchar,
        settings -> Json,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    service_permissions (id) {
        id -> Uuid,
        service_name -> Varchar,
        tenant_id -> Uuid,
        database_url -> Varchar,
        allowed_operations -> Array<Text>,
        resource_limits -> Json,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(event_dead_letter_queue -> event_store (original_event_id));
diesel::joinable!(event_projections -> event_store (last_processed_event_id));
diesel::joinable!(service_permissions -> tenants (tenant_id));

diesel::allow_tables_to_appear_in_same_query!(
    events,
    agents,
    event_store,
    event_snapshots,
    event_subscriptions,
    event_dead_letter_queue,
    event_projections,
    users,
    tenants,
    service_permissions,
);
