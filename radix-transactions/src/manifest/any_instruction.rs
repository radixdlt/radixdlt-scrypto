use crate::internal_prelude::*;
use decompiler::*;

/// A type representing an enum of all possible instructions.
/// This can then be mapped into a specific instruction type.
pub type AnyInstruction = InstructionV2;

/// A marker trait for an Instruction set, e.g. InstructionV1
pub trait ManifestInstructionSet: TryFrom<AnyInstruction> + Into<AnyInstruction> + Clone {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        self.map_ref(context)
    }

    fn effect(&self) -> ManifestInstructionEffect {
        self.map_ref(EffectMapper)
    }

    fn into_any(self) -> AnyInstruction {
        self.map_self(IntoAnyMapper)
    }

    fn try_convert<T>(self) -> Result<T, <AnyInstruction as TryInto<T>>::Error>
    where
        AnyInstruction: TryInto<T>,
    {
        self.map_self(IntoThroughAnyMapper(PhantomData))
    }

    fn map_ref<M: InstructionRefMapper>(&self, mapper: M) -> M::Output<'_>;
    fn map_self<M: OwnedInstructionMapper>(self, mapper: M) -> M::Output;
}

/// This trait is intended to reduce the boilerplate of defining actions which can
/// be applied to all instructions in a set.
pub trait InstructionRefMapper {
    type Output<'i>;
    fn apply<'i>(self, instruction: &'i impl ManifestInstruction) -> Self::Output<'i>;
}

impl<'a, 'b> InstructionRefMapper for &'b mut DecompilationContext<'a> {
    type Output<'i> = Result<DecompiledInstruction, DecompileError>;

    fn apply<'i>(self, instruction: &'i impl ManifestInstruction) -> Self::Output<'i> {
        instruction.decompile(self)
    }
}

struct EffectMapper;
impl InstructionRefMapper for EffectMapper {
    type Output<'i> = ManifestInstructionEffect<'i>;

    fn apply<'i>(self, instruction: &'i impl ManifestInstruction) -> Self::Output<'i> {
        instruction.effect()
    }
}

pub trait OwnedInstructionMapper {
    type Output;
    fn apply(self, instruction: impl ManifestInstruction) -> Self::Output;
}

struct IntoAnyMapper;
impl OwnedInstructionMapper for IntoAnyMapper {
    type Output = AnyInstruction;

    fn apply(self, instruction: impl ManifestInstruction) -> AnyInstruction {
        instruction.into_any()
    }
}

struct IntoThroughAnyMapper<T>(PhantomData<T>)
where
    AnyInstruction: TryInto<T>;
impl<T> OwnedInstructionMapper for IntoThroughAnyMapper<T>
where
    AnyInstruction: TryInto<T>,
{
    type Output = Result<T, <AnyInstruction as TryInto<T>>::Error>;

    fn apply(self, instruction: impl ManifestInstruction) -> Self::Output {
        <AnyInstruction as TryInto<T>>::try_into(instruction.into_any())
    }
}
