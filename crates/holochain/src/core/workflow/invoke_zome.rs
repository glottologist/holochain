use super::{
    error::WorkflowResult, error::WorkflowRunResult, WorkflowCaller, WorkflowEffects,
    WorkflowTriggers,
};
use crate::core::{ribosome::RibosomeT, state::workspace::InvokeZomeWorkspace};
use futures::future::FutureExt;
use holochain_state::{db::DbManager, prelude::Reader};
use holochain_types::{nucleus::ZomeInvocation, prelude::Todo};
use must_future::MustBoxFuture;

pub type ZomeInvocationResult = Todo;

pub struct InvokeZomeWorkflow<'env, Ribosome: RibosomeT + 'env> {
    ribosome: Ribosome,
    invocation: ZomeInvocation,
    __lifetime: std::marker::PhantomData<&'env ()>,
}

pub struct InvokeZomeTriggers;
impl WorkflowTriggers for InvokeZomeTriggers {}

impl<'env, Ribosome: RibosomeT + Send + Sync> WorkflowCaller<'env>
    for InvokeZomeWorkflow<'env, Ribosome>
{
    type Output = ZomeInvocationResult;
    type Workspace = InvokeZomeWorkspace<'env>;
    type Triggers = InvokeZomeTriggers;

    fn workflow(
        self,
        workspace: Self::Workspace,
    ) -> MustBoxFuture<'env, WorkflowResult<'env, Self::Output, Self>> {
        invoke_zome(workspace, self.ribosome, self.invocation)
            .boxed()
            .into()
    }
}

async fn invoke_zome<'env, Ribosome: RibosomeT + Send + Sync + 'env>(
    workspace: InvokeZomeWorkspace<'env>,
    _ribosome: Ribosome,
    _invocation: ZomeInvocation,
) -> WorkflowResult<'env, ZomeInvocationResult, InvokeZomeWorkflow<'env, Ribosome>> {
    let fx = WorkflowEffects {
        workspace,
        triggers: todo!(""),
        signals: Default::default(),
        callbacks: Default::default(),
        __lifetime: std::marker::PhantomData,
    };
    let result = todo!("this will be the actual zome function return value");
    Ok((result, fx))
}

#[cfg(test_TODO_FIX)]
pub mod tests {
    use super::*;
    use crate::{
        agent::{source_chain::tests::test_initialized_chain, SourceChainCommitBundle},
        conductor_api::MockCellConductorApi,
        ribosome::MockRibosomeT,
        test_utils::fake_cell_id,
    };
    use holochain_types::{entry::Entry, error::SkunkResult};
    use tempdir::TempDir;

    #[tokio::test]
    async fn can_invoke_zome_with_mock() {
        let cell_id = fake_cell_id("mario");
        let tmpdir = TempDir::new("holochain_2020").unwrap();
        let persistence = SourceChainPersistence::test(tmpdir.path());
        let chain = test_initialized_chain("mario", &persistence);
        let invocation = ZomeInvocation {
            cell_id: cell_id.clone(),
            zome_name: "zome".into(),
            fn_name: "fn".into(),
            as_at: "KwyXHisn".into(),
            args: "args".into(),
            provenance: cell_id.agent_id().to_owned(),
            cap: CapabilityRequest,
        };

        let mut ribosome = MockRibosomeT::new();
        ribosome
            .expect_call_zome_function()
            .times(1)
            .returning(|bundle, _| Ok(ZomeInvocationResponse));

        // TODO: make actual assertions on the conductor_api, once more of the
        // actual logic is fleshed out
        let mut conductor_api = MockCellConductorApi::new();

        let result = invoke_zome(invocation, chain, ribosome, conductor_api).await;
        assert!(result.is_ok());
    }

    // TODO: can try making a fake (not mock) ribosome that has some hard-coded logic
    // for calling into a ZomeApi, rather than needing to write a test DNA. This will
    // have to wait until the whole WasmRibosome thing is more fleshed out.
    // struct FakeRibosome;

    // impl RibosomeT for FakeRibosome {
    //     fn run_validation(self, cursor: &source_chain::Cursor, entry: Entry) -> ValidationResult {
    //         unimplemented!()
    //     }

    //     /// Runs the specified zome fn. Returns the cursor used by HDK,
    //     /// so that it can be passed on to source chain manager for transactional writes
    //     fn call_zome_function(
    //         self,
    //         bundle: SourceChainCommitBundle,
    //         invocation: ZomeInvocation,
    //     ) -> SkunkResult<(ZomeInvocationResponse, SourceChainCommitBundle)> {
    //         unimplemented!()
    //     }
    // }
}
