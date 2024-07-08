// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::prelude::*;
use extend::ext;

#[ext]
pub impl DefaultLedgerSimulator {
    fn execute_manifest_without_auth(
        &mut self,
        manifest: TransactionManifestV1,
    ) -> TransactionReceiptV1 {
        self.execute_manifest_with_enabled_modules(
            manifest,
            EnabledModules::for_test_transaction() & !EnabledModules::AUTH,
        )
    }

    fn execute_manifest_with_enabled_modules(
        &mut self,
        manifest: TransactionManifestV1,
        enabled_modules: EnabledModules,
    ) -> TransactionReceiptV1 {
        let mut execution_config = ExecutionConfig::for_test_transaction();
        execution_config.system_overrides = Some(SystemOverrides {
            disable_costing: !enabled_modules.contains(EnabledModules::COSTING),
            disable_limits: !enabled_modules.contains(EnabledModules::LIMITS),
            disable_auth: !enabled_modules.contains(EnabledModules::AUTH),
            network_definition: Default::default(),
            costing_parameters: Default::default(),
            limit_parameters: Default::default(),
        });
        execution_config.enable_kernel_trace =
            enabled_modules.contains(EnabledModules::KERNEL_TRACE);
        execution_config.enable_cost_breakdown =
            enabled_modules.contains(EnabledModules::KERNEL_TRACE);
        execution_config.execution_trace =
            if enabled_modules.contains(EnabledModules::EXECUTION_TRACE) {
                Some(1)
            } else {
                None
            };

        let nonce = self.next_transaction_nonce();
        let test_transaction = TestTransaction::new_from_nonce(manifest, nonce);
        let prepared_transaction = test_transaction.prepare().unwrap();
        let executable =
            prepared_transaction.get_executable(Default::default());
        self.execute_transaction(executable, execution_config)
    }

    /// Constructs a notarized transaction and executes it. This is primarily
    /// used in the testing of fees to make sure that they're approximated in
    /// the best way.
    fn construct_and_execute_notarized_transaction(
        &mut self,
        manifest: TransactionManifestV1,
        notary_private_key: &PrivateKey,
    ) -> TransactionReceiptV1 {
        let network_definition = NetworkDefinition::simulator();
        let current_epoch = self.get_current_epoch();
        let transaction = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: network_definition.id,
                start_epoch_inclusive: current_epoch,
                end_epoch_exclusive: current_epoch.after(10).unwrap(),
                nonce: self.next_transaction_nonce(),
                notary_public_key: notary_private_key.public_key(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(manifest)
            .notarize(notary_private_key)
            .build();
        self.execute_notarized_transaction(&transaction.to_raw().unwrap())
    }
}
