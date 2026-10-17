//! Compiled PostgreSQL 17 deparser forms of the fixed policy meanings.
//! Keep full operators, casts and grouping; normalize whitespace only.

pub(crate) const TABLES: [&str; 13] = [
    "organizations",
    "domains",
    "principals",
    "memberships",
    "namespace_guard",
    "workspace_policies",
    "source_policies",
    "resources",
    "party_aliases",
    "grants",
    "immutable_chains",
    "immutable_members",
    "qualification_markers",
];

#[derive(Eq, PartialEq)]
pub(crate) struct Policy {
    pub table: String,
    pub name: String,
    pub command: String,
    pub permissive: bool,
    pub roles: Vec<String>,
    pub using: Option<String>,
    pub check: Option<String>,
}

pub(crate) fn normalized(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn setting(key: &str) -> String {
    format!("NULLIF(current_setting('arboresce.{key}'::text, true), ''::text)")
}
fn equal(column: &str, key: &str) -> String {
    format!("(({column})::text = {})", setting(key))
}
fn conjunction(terms: Vec<String>) -> String {
    format!("({})", terms.join(" AND "))
}
fn either(a: String, b: String) -> String {
    format!("({a} OR {b})")
}
fn uuid_equal(a: &str, b: &str) -> String {
    format!("(({a})::uuid = ({b})::uuid)")
}
fn exists(from: &str, predicate: String) -> String {
    format!("(EXISTS ( SELECT 1 FROM {from} WHERE {predicate}))")
}

fn context() -> String {
    conjunction(
        ["organization_id", "domain_id", "principal_id"]
            .iter()
            .map(|key| format!("({} IS NOT NULL)", setting(key)))
            .collect(),
    )
}
fn tenant() -> String {
    conjunction(vec![
        equal("organization_id", "organization_id"),
        equal("domain_id", "domain_id"),
    ])
}

fn participant(table: &str) -> String {
    let outer = format!("{table}.principal_id");
    let predicate = conjunction(vec![
        equal("im.organization_id", "organization_id"),
        equal("im.domain_id", "domain_id"),
        equal("im.chain_id", "chain_id"),
        either(
            uuid_equal(&outer, "im.issuer_id"),
            uuid_equal(&outer, "im.subject_id"),
        ),
    ]);
    either(
        equal("principal_id", "principal_id"),
        exists("arboresce_authority.immutable_members im", predicate),
    )
}

fn selected(table: &str) -> String {
    match table {
        "namespace_guard" => context(),
        "organizations" => equal("organization_id", "organization_id"),
        "principals" => participant(table),
        "memberships" => conjunction(vec![
            equal("organization_id", "organization_id"),
            participant(table),
        ]),
        "source_policies" => exists(
            "arboresce_authority.workspace_policies wp",
            conjunction(vec![
                equal("wp.organization_id", "organization_id"),
                equal("wp.domain_id", "domain_id"),
                "(source_policies.policy_id = ANY (wp.source_policy_ids))".to_owned(),
            ]),
        ),
        "qualification_markers" => conjunction(vec![
            equal("organization_id", "organization_id"),
            equal("domain_id", "domain_id"),
            equal("executor_principal_id", "principal_id"),
        ]),
        _ => tenant(),
    }
}

fn policy(
    table: &str,
    name: &str,
    command: &str,
    permissive: bool,
    roles: &[&str],
    using: Option<String>,
    check: Option<String>,
) -> Policy {
    Policy {
        table: table.to_owned(),
        name: name.to_owned(),
        command: command.to_owned(),
        permissive,
        roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        using: using.map(|s| normalized(&s)),
        check: check.map(|s| normalized(&s)),
    }
}

pub(crate) fn expected() -> Vec<Policy> {
    let admission = ["harness_authority_admission"];
    let permission = ["harness_authority_permission"];
    let both = [admission[0], permission[0]];
    let mut policies = Vec::new();
    for table in TABLES {
        policies.push(policy(
            table,
            "internal_complete_context",
            "*",
            false,
            &both,
            Some(context()),
            Some(context()),
        ));
        policies.push(policy(
            table,
            "internal_selected_read",
            "r",
            true,
            if table == "qualification_markers" {
                &admission
            } else {
                &both
            },
            Some(selected(table)),
            None,
        ));
        if !matches!(table, "party_aliases" | "qualification_markers") {
            let admission_lock = table != "resources";
            let permission_lock = matches!(
                table,
                "organizations"
                    | "domains"
                    | "principals"
                    | "source_policies"
                    | "immutable_chains"
                    | "immutable_members"
            );
            policies.push(policy(
                table,
                "admission_selected_lock_or_control",
                "w",
                true,
                &admission,
                Some(selected(table)),
                Some(if admission_lock {
                    "false".to_owned()
                } else {
                    selected(table)
                }),
            ));
            policies.push(policy(
                table,
                "permission_selected_lock_or_change",
                "w",
                true,
                &permission,
                Some(selected(table)),
                Some(if permission_lock {
                    "false".to_owned()
                } else {
                    selected(table)
                }),
            ));
            if admission_lock {
                policies.push(policy(
                    table,
                    "admission_lock_only_check",
                    "w",
                    false,
                    &admission,
                    Some(selected(table)),
                    Some("false".to_owned()),
                ));
            }
            if permission_lock {
                policies.push(policy(
                    table,
                    "permission_lock_only_check",
                    "w",
                    false,
                    &permission,
                    Some(selected(table)),
                    Some("false".to_owned()),
                ));
            }
        }
    }
    policies.push(policy(
        "qualification_markers",
        "admission_scoped_marker",
        "a",
        true,
        &admission,
        None,
        Some(selected("qualification_markers")),
    ));
    policies.push(policy(
        "principals",
        "principal_context",
        "r",
        true,
        &["public"],
        Some(equal("principal_id", "principal_id")),
        None,
    ));
    policies.push(policy(
        "memberships",
        "membership_context",
        "r",
        true,
        &["public"],
        Some(conjunction(vec![
            equal("organization_id", "organization_id"),
            equal("principal_id", "principal_id"),
        ])),
        None,
    ));
    let org = conjunction(vec![
        equal("organization_id", "organization_id"),
        exists(
            "(arboresce_authority.memberships m JOIN arboresce_authority.principals p USING (principal_id))",
            conjunction(vec![
                uuid_equal("m.organization_id", "organizations.organization_id"),
                "(m.state = 'active'::text)".to_owned(),
                "(p.state = 'active'::text)".to_owned(),
            ]),
        ),
    ]);
    policies.push(policy(
        "organizations",
        "organization_context",
        "r",
        true,
        &["public"],
        Some(org),
        None,
    ));
    let domain = conjunction(vec![
        equal("organization_id", "organization_id"),
        equal("domain_id", "domain_id"),
        exists(
            "((arboresce_authority.organizations o JOIN arboresce_authority.memberships m USING (organization_id)) JOIN arboresce_authority.principals p USING (principal_id))",
            conjunction(vec![
                uuid_equal("o.organization_id", "domains.organization_id"),
                uuid_equal("m.domain_id", "domains.domain_id"),
                "(o.state = 'active'::text)".to_owned(),
                "(m.state = 'active'::text)".to_owned(),
                "(p.state = 'active'::text)".to_owned(),
            ]),
        ),
    ]);
    policies.push(policy(
        "domains",
        "domain_context",
        "r",
        true,
        &["public"],
        Some(domain),
        None,
    ));
    policies.sort_by(|a, b| (&a.table, &a.name).cmp(&(&b.table, &b.name)));
    policies
}
