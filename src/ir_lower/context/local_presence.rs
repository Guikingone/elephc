//! Purpose:
//! Tracks PHP local binding presence independently from inferred value storage.
//! Adds ordinary EIR state only for locals observed by a silent existence probe.
//!
//! Called from:
//! - Lazy local isset lowering and final function-body lowering.
//!
//! Key details:
//! - Flags start in entry, before any throwing RHS, and become set only after a store.
//! - Clearing a binding clears its flag before a destructor can throw.
//! - Explicit EIR loads/stores keep optimizer liveness and every target backend coherent.

use super::*;
use crate::ir::{InstId,Instruction,Value,ValueDef};

impl LoweringContext<'_, '_> {
    /// Loads an explicit presence flag for a normal local, leaving external storage to its own probe.
    pub(crate) fn local_presence_flag(&mut self,name:&str,span:Span)->Option<LoweredValue> {
        if self.local_uses_global_storage(name)
            || self.extern_globals.contains_key(name)
            || self.eval_scope_read_param.is_some()
            || crate::names::is_generated_local_name(name)
        {return None;}
        let slot=self.declare_local(name,self.local_type(name));
        if self.builder.local_kind(slot)!=LocalKind::PhpLocal {return None;}
        let flag=if let Some(flag)=self.presence_flags.get(&slot) {*flag}else{
            let flag=self.builder.add_local(Some(format!("{name}#presence")),IrType::I64,PhpType::Int,LocalKind::HiddenTemp);
            let initialized=self.builder.function().params.get(slot.as_raw() as usize)
                .is_some_and(|param|param.name==name);
            let entry=self.builder.function().entry;
            let count=self.builder.function().blocks[entry.as_raw() as usize].instructions.len();
            self.builder.seed_entry_int_local(flag,i64::from(initialized));
            self.presence_flag_seeds.extend_from_slice(&self.builder.function().blocks[entry.as_raw() as usize].instructions[count..]);
            self.presence_flags.insert(slot,flag);flag
        };
        Some(self.emit_value(Op::LoadLocal,Vec::new(),Some(Immediate::LocalSlot(flag)),PhpType::Int,Op::LoadLocal.default_effects(),Some(span)))
    }

    /// Gives a probe-only placeholder its first real storage type before any store widens it.
    /// Merely observing an absent binding must not turn a later string reference capture into
    /// a Mixed payload. A true null store marks the slot initialized and retains normal widening.
    pub(crate) fn specialize_unstored_presence_slot(&mut self, slot: LocalSlotId, php_type: &PhpType) {
        if !self.presence_flags.contains_key(&slot)
            || self.initialized_slots.contains(&slot)
            || self.builder.local_php_type(slot) != PhpType::Void
            || matches!(php_type.codegen_repr(), PhpType::Void | PhpType::Never)
        {
            return;
        }
        let mut function = self.builder.snapshot_function();
        let local = &mut function.locals[slot.as_raw() as usize];
        local.php_type = php_type.clone();
        local.ir_type = value_ir_type(php_type);
        self.builder.restore_function(function);
    }

    /// Defers a newly declared null probe slot's type until loop-carried stores are known.
    /// The presence branch prevents loading the slot before its first real store. An abandoned
    /// name starts as Void in the source-order environment, but its new slot can carry a later
    /// iteration's value. Only this dedicated probe load changes, never an ordinary PHP read.
    pub(crate) fn record_presence_probe_load(&mut self, name: &str, value: LoweredValue) {
        let Some(slot) = self.local_slots.get(name).copied() else { return; };
        if !self.initialized_slots.contains(&slot)
            && self.builder.value_php_type(value.value) == PhpType::Void
            && self.builder.value_defining_op(value.value) == Some(Op::LoadLocal)
        {
            self.presence_probe_loads.push((value.value, slot));
        }
    }

    /// Materializes binding updates after all source-order stores and speculative rollbacks are known.
    pub(crate) fn finalize_local_presence(&mut self) {
        if self.presence_flags.is_empty(){return;}
        let mut function=self.builder.snapshot_function();
        for &(value, slot) in &self.presence_probe_loads {
            let php_type = function.locals[slot.as_raw() as usize].php_type.clone();
            let ir_type = value_ir_type(&php_type);
            let ownership = if Ownership::php_type_needs_lifetime_tracking(&php_type) { Ownership::Borrowed } else { Ownership::NonHeap };
            let definition = &mut function.values[value.as_raw() as usize];
            definition.php_type = php_type.clone();
            definition.ir_type = ir_type;
            definition.ownership = ownership;
            if let ValueDef::Instruction { inst, .. } = definition.def {
                let instruction = &mut function.instructions[inst.as_raw() as usize];
                instruction.result_php_type = php_type;
                instruction.result_type = ir_type;
                instruction.result_ownership = ownership;
            }
        }
        let seeds=self.presence_flag_seeds.iter().copied().collect::<HashSet<_>>();
        let entry=function.entry.as_raw() as usize;
        function.blocks[entry].instructions.retain(|id|!seeds.contains(id));
        let (zero_inst,zero)=constant(&mut function,0);
        let (one_inst,one)=constant(&mut function,1);
        for index in 0..function.blocks.len() {
            let old=std::mem::take(&mut function.blocks[index].instructions);
            let mut updated=Vec::new();
            if index==entry {
                updated.extend([zero_inst,one_inst]);
                updated.extend(self.presence_flag_seeds.iter().copied());
            }
            for id in old {
                let inst=&function.instructions[id.as_raw() as usize];
                let write=local_binding_write(inst).and_then(|(slot,clears)|self.presence_flags.get(&slot).map(|flag|(*flag,clears)));
                let span=inst.span;
                if let Some((flag,true))=write {updated.push(flag_store(&mut function,flag,zero,span));}
                updated.push(id);
                if let Some((flag,false))=write {updated.push(flag_store(&mut function,flag,one,span));}
            }
            function.blocks[index].instructions=updated;
        }
        // Entry insertion and post-store writes change table positions but not instruction IDs.
        for block in &function.blocks {
            for (index,id) in block.instructions.iter().copied().enumerate() {
                if let Some(value)=function.instructions[id.as_raw() as usize].result {
                    function.values[value.as_raw() as usize].def=ValueDef::Instruction{block:block.id,index:index as u32,inst:id};
                }
            }
        }
        self.builder.restore_function(function);
    }
}

/// Identifies writes to a PHP binding rather than incidental pointer/owner reads.
fn local_binding_write(inst:&Instruction)->Option<(LocalSlotId,bool)> {
    let slot=match inst.immediate {
        Some(Immediate::LocalSlot(slot))=>slot,
        Some(Immediate::LocalSlotPair{first,..}) if matches!(inst.op,Op::PromoteLocalRefCell|Op::AliasLocalRefCell|Op::BindRefCellPtr|Op::AdoptRefCellPtr)=>first,
        _=>return None,
    };
    match inst.op {
        Op::UnsetLocal|Op::ZeroLocalSlot|Op::ReleaseLocalSlot=>Some((slot,true)),
        Op::StoreLocal|Op::StoreRefCell|Op::PromoteLocalRefCell|Op::AliasLocalRefCell|Op::BindRefCellPtr|Op::AdoptRefCellPtr|Op::StrAppend=>Some((slot,false)),
        _=>None,
    }
}

/// Adds a nonheap flag constant whose final entry position is repaired with the other definitions.
fn constant(function:&mut Function,value:i64)->(InstId,ValueId) {
    let inst=InstId::from_raw(function.instructions.len() as u32);let result=ValueId::from_raw(function.values.len() as u32);
    function.values.push(Value{ir_type:IrType::I64,php_type:PhpType::Int,def:ValueDef::Instruction{block:function.entry,index:0,inst},ownership:Ownership::NonHeap});
    function.instructions.push(Instruction::new(Op::ConstI64,Vec::new(),Some(Immediate::I64(value)),Some(result),IrType::I64,PhpType::Int,Ownership::NonHeap,Op::ConstI64.default_effects(),None));
    (inst,result)
}

/// Adds an ordinary visible-to-optimization EIR flag store.
fn flag_store(function:&mut Function,slot:LocalSlotId,value:ValueId,span:Option<Span>)->InstId {
    let id=InstId::from_raw(function.instructions.len() as u32);
    function.instructions.push(Instruction::new(Op::StoreLocal,vec![value],Some(Immediate::LocalSlot(slot)),None,IrType::Void,PhpType::Void,Ownership::NonHeap,Op::StoreLocal.default_effects(),span));id
}
