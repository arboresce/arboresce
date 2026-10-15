# Aliased source inclusion

These eight original public synthetic cases use MIT OR Apache-2.0. The initial
complete-source profile rejects source include! in every layer, including when
an aliased root and renamed import hide its invocation. Importing the terminal
name include is conservatively rejected for any root in this initial profile;
this does not require arbitrary name resolution. Raw names have the same
identity. Text and byte resource aliases remain permitted. The unknown-root
case fixes this conservative policy explicitly rather than assuming a resolved
macro definition. These are syntax-policy inputs, not actual runtime evidence.
Independent expected review precedes consumption. The prior 210 cases and
governing files remain unchanged.
