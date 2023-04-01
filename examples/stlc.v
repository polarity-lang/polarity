Require Import Coq.Arith.PeanoNat.

Inductive Exp :=
    | Var: nat -> Exp
    | Lam: Exp -> Exp
    | App: Exp -> Exp -> Exp.

Inductive Typ :=
    | FunT : Typ -> Typ -> Typ
    | VarT : nat -> Typ.

Inductive Ctx :=
    | Nil
    | Cons(t: Typ)(ts: Ctx).

Fixpoint append(ctx1 ctx2: Ctx): Ctx :=
    match ctx1 with
    | Nil => ctx2
    | Cons t ts => Cons t (append ts ctx2)
    end.

Fixpoint len(ctx: Ctx): nat :=
    match ctx with
    | Nil => 0
    | Cons t ts => S (len ts)
    end.

Fixpoint subst(e: Exp)(v: nat)(by_e: Exp): Exp :=
    match e with
    | Var x =>
        if Nat.eqb x v then
            by_e
        else
            Var (if Nat.leb v x then (pred x) else x)
    | Lam e => Lam (subst e (S v) by_e)
    | App e1 e2 => App (subst e1 v by_e) (subst e2 v by_e)
    end.

Inductive Elem: nat -> Typ -> Ctx -> Prop :=
    | Here(t: Typ)(ts: Ctx): (Elem 0 t (Cons t ts))
    | There(x: nat)(t t2: Typ)(ts: Ctx)(prf: Elem x t ts): Elem (S x) t (Cons t2 ts).

Inductive HasType: Ctx -> Exp -> Typ -> Prop :=
    | TVar(ctx: Ctx)(x: nat)(t: Typ)(elem: Elem x t ctx): HasType ctx (Var x) t
    | TLam(ctx: Ctx)(t1 t2: Typ)(e: Exp)(body: HasType (Cons t1 ctx) e t2): HasType ctx (Lam e) (FunT t1 t2)
    | TApp(ctx: Ctx)(t1 t2: Typ)(e1 e2: Exp)
        (e1_t: HasType ctx e1 (FunT t1 t2))
        (e2_t: HasType ctx e2 t1): HasType ctx (App e1 e2) t2.

Inductive Eval: Exp -> Exp -> Prop :=
    | EBeta(e1 e2: Exp): Eval (App (Lam e1) e2) (subst e1 0 e2)
    | ECongApp1(e1 e1': Exp)(h: Eval e1 e1')(e2: Exp): Eval (App e1 e2) (App e1' e2)
    | ECongApp2(e1 e2 e2': Exp)(h: Eval e2 e2'): Eval (App e1 e2) (App e1 e2').

Inductive IsValue : Exp -> Prop :=
    | VLam(e: Exp): IsValue (Lam e).

Inductive Progress : Exp -> Prop :=
    | PVal(e: Exp)(h: IsValue e): Progress e
    | PStep(e1 e2: Exp)(h: Eval e1 e2): Progress e1.

Theorem progress: forall (self: Exp)(t: Typ)(h: HasType Nil self t), Progress self.
Proof.
    induction self; intros.
    - inversion h; subst. inversion elem.
    - repeat constructor.
    - inversion h; subst. inversion e1_t; subst.
      + inversion elem.
      + eapply PStep. apply EBeta.
      + pose proof (IHself1 (FunT t1 t) e1_t) as H.
        inversion H; subst.
        * inversion h0.
        * apply PStep with (e2 := (App e3 self2)).
          eapply ECongApp1. assumption.
Qed.

Lemma ctx_unique: forall (t1 t2: Typ) (ctx: Ctx), Elem 0 t1 (Cons t2 ctx) -> t1 = t2.
Proof.
    intros. inversion H; subst. reflexivity.
Qed.

Lemma ctx_lookup: forall (ctx1 ctx2: Ctx)(t1 t2: Typ), Elem (len ctx1) t1 (append ctx1 (Cons t2 ctx2)) -> t1 = t2.
Proof.
    induction ctx1; intros.
    - simpl in H. eapply ctx_unique. eassumption.
    - simpl in *. eapply IHctx1. inversion H; subst. eassumption.
Qed.

Lemma append_nil: forall ctx, append ctx Nil = ctx.
Proof.
    induction ctx.
    - reflexivity.
    - simpl. rewrite IHctx. reflexivity.
Qed.

Lemma append_assoc: forall (ctx1 ctx2 ctx3: Ctx), append (append ctx1 ctx2) ctx3 = append ctx1 (append ctx2 ctx3) .
Proof.
    induction ctx1; intros.
    - reflexivity.
    - simpl. rewrite IHctx1. reflexivity.
Qed.

Lemma elem_append: forall n t t2 ctx, Elem n t ctx -> Elem n t (append ctx (Cons t2 Nil)).
Proof.
    induction n; intros.
    - inversion H; subst. constructor.
    - inversion H; subst.
      constructor. fold append.
      apply IHn. assumption.
Qed.

Lemma weaken': forall (e: Exp)(ctx1: Ctx)(t t2: Typ)(h: HasType ctx1 e t), HasType (append ctx1 (Cons t2 Nil)) e t.
Proof.
    induction e; intros.
    - inversion h; subst.
      constructor.
      apply elem_append.
      assumption.
    - inversion h; subst.
      pose proof (IHe (Cons t1 ctx1) t0 t2) body.
      constructor.
      assumption.
    - inversion h; subst.
      econstructor.
      + eapply IHe1; eassumption.
      + eapply IHe2; eassumption.
Qed.

Lemma weaken'': forall (ctx2 ctx1: Ctx)(e: Exp)(t: Typ)(h: HasType ctx1 e t), HasType (append ctx1 ctx2) e t.
Proof.
    induction ctx2; intros.
    - rewrite append_nil. assumption.
    - eapply weaken' in h.
      pose proof (IHctx2 _ e t0 h).
      rewrite append_assoc in H. simpl in *.
      eassumption.
Qed.

Lemma weaken: forall (ctx: Ctx)(e: Exp)(t: Typ)(h: HasType Nil e t), HasType ctx e t.
Proof.
    intros.
    apply (weaken'' ctx Nil e t h).
Qed.

Search (S _ < S _).

Lemma elem_append_lt: forall (n: nat) (ctx1 ctx2 ctx3: Ctx) (t: Typ),
    n < len ctx1 -> Elem n t (append ctx1 ctx2) -> Elem n t (append ctx1 ctx3).
Proof.
    induction n; intros.
    - inversion H0; subst. destruct ctx1.
      + inversion H.
      + simpl. apply ctx_unique in H0; subst. constructor.
    - inversion H0; subst. destruct ctx1.
      + inversion H.
      + simpl. constructor. eapply IHn.
        * simpl in H. rewrite <- Nat.succ_lt_mono in H. assumption.
        * inversion H0; subst. eassumption.
Qed.

Lemma elem_append_shift: forall (ctx1 ctx2: Ctx) (t t2: Typ) (n: nat),
    len ctx1 < n -> Elem n t (append ctx1 (Cons t2 ctx2))
    -> Elem (pred n) t (append ctx1 ctx2).
Proof.
    induction ctx1; intros.
    - simpl in *. destruct n.
      + inversion H.
      + simpl. inversion H0; subst. assumption.
    - simpl. destruct n. simpl in *.
      + inversion H.
      + destruct n.
        * inversion H; subst. inversion H2.
        * assert (S n <> 0). discriminate.
          rewrite <- (Nat.succ_pred (S n) H1).
          simpl.
          constructor.
          simpl in H0. inversion H0; subst.
          eapply IHctx1 with (n := S n).
          -- simpl in H. rewrite <- Nat.succ_lt_mono in H. assumption.
          -- eassumption.
Qed.

Theorem subst_lemma2: forall (e: Exp)(ctx1 ctx2: Ctx)(t1 t2: Typ)(by_e: Exp)
    (h_e: HasType (append ctx1 (Cons t1 ctx2)) e t2)
    (h_by: HasType Nil by_e t1),
    HasType (append ctx1 ctx2) (subst e (len ctx1) by_e) t2.
Proof.
    induction e; intros.
    - unfold subst.
      destruct (Nat.eqb n (len ctx1)) eqn:E.
      + inversion h_e; subst. apply Nat.eqb_eq in E; subst.
        apply ctx_lookup in elem; subst.
        eapply weaken in h_by.
        apply h_by.
      + apply Nat.eqb_neq in E.
        destruct (Nat.leb (len ctx1) n) eqn:E2.
        * constructor.
          apply (elem_append_shift ctx1 ctx2 t2 t1 n).
          -- apply Nat.leb_le in E2.
             destruct E2.
             ++ exfalso. apply E. reflexivity.
             ++ unfold "<". rewrite <- Nat.succ_le_mono. assumption.
          -- inversion h_e; subst. assumption.
        * constructor.
          apply (elem_append_lt n ctx1 (Cons t1 ctx2) ctx2 t2).
          -- apply Nat.leb_gt. assumption.
          -- inversion h_e; subst. assumption.
    - simpl. inversion h_e; subst.
      constructor.
      pose proof (IHe (Cons t0 ctx1) ctx2) t1 t3 by_e body h_by.
      simpl in H. assumption.
    - simpl. inversion h_e; subst. econstructor.
      + eapply IHe1; eassumption.
      + eapply IHe2; eassumption.
Qed.

Theorem subst_lemma: forall (e: Exp)(ctx: Ctx)(t1 t2: Typ)(by_e: Exp)
    (h_e: HasType (Cons t1 ctx) e t2)
    (h_by: HasType Nil by_e t1),
    HasType ctx (subst e 0 by_e) t2.
Proof.
    intros.
    apply (subst_lemma2 e Nil ctx t1 t2 by_e h_e h_by).
Qed.

Theorem preservation: forall (self e2: Exp)(t: Typ)(h_t: HasType Nil self t)(h_eval: Eval self e2), HasType Nil e2 t.
Proof.
    induction self; intros.
    - inversion h_eval.
    - inversion h_t; subst. inversion h_eval; subst.
    - inversion h_eval; subst.
        + inversion h_t; subst.  inversion e1_t; subst. eapply subst_lemma; eassumption.
        + inversion h_t; subst. econstructor.
            * eapply IHself1; eassumption.
            * eassumption.
        + inversion h_t; subst. econstructor.
            * eassumption.
            * eapply IHself2; eassumption.
Qed.
