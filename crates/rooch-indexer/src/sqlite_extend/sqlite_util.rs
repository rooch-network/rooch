// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

fn to_sql(&self, out: &mut DB::QueryBuilder, backend: &DB) -> QueryResult<()> {
    let mut options = AstPassToSqlOptions::default();
    self.walk_ast(AstPass::to_sql(out, &mut options, backend))
}