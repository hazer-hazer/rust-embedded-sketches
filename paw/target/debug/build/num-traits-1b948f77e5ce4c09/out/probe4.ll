; ModuleID = 'probe4.4c45227cdc1ee8e0-cgu.0'
source_filename = "probe4.4c45227cdc1ee8e0-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@alloc_cb4327a7e57a011f0750dae0d8bd30df = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"/home/rust/rust/library/core/src/num/mod.rs" }>, align 1
@alloc_c6d699b5ce40adf63c462c8d94ba8466 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_cb4327a7e57a011f0750dae0d8bd30df, [16 x i8] c"+\00\00\00\00\00\00\00y\04\00\00\05\00\00\00" }>, align 8
@str.0 = internal unnamed_addr constant [25 x i8] c"attempt to divide by zero"

; probe4::probe
; Function Attrs: nonlazybind uwtable
define void @_ZN6probe45probe17h7688148e85145a99E() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false)
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h1b9b1422707c0943E.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17hde10d641a9870f24E(ptr align 1 @str.0, i64 25, ptr align 8 @alloc_c6d699b5ce40adf63c462c8d94ba8466) #3
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h1b9b1422707c0943E.exit": ; preds = %start
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn nonlazybind uwtable
declare void @_ZN4core9panicking5panic17hde10d641a9870f24E(ptr align 1, i64, ptr align 8) unnamed_addr #2

attributes #0 = { nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(none) }
attributes #2 = { cold noinline noreturn nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #3 = { noreturn }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{!"rustc version 1.76.0-nightly (88269fa9e 2024-02-09) (1.76.0.1)"}
