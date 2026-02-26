# Polarity for Haskellers

It might be a bit confusing to understand what Polarity's `data` and `codata` and `def` and `codef` do differently than Haskell's data. Especially observing infinite streams is incredibly easy in Haskell.

Take for example:

```haskell
data Stream a = MkStream a (Stream a)

ones :: Stream Int
ones = MkStream 1 ones
```
Now, we could easily just observe the 5th element by peeling off 5 `MkStream` constructors and returning the the element in the cell contained there; no need for `codata`.
However, it is not that easy in typical strict languages. For that, let us make Haskell a language with strict datatypes, using the `-XUnliftedDataTypes` extension.

```haskell
import Data.Kind (Type)
import GHC.Exts (Int#, UnliftedType, (-#))
import qualified GHC.Int as Int

type SType = UnliftedType
type LType = Type

-- strict Ints, wrapping machine Ints
type I :: SType
data I = I# Int#

-- strict, finite lists
type L :: SType -> SType
data L a = N | C a (L a)
```

`SType`s are always strict, that is, they will never be thunks. That means, that our usual definition of a `Stream` falls through; as soon as we try to observe `ones`, our interpreter will hang. Try it out for yourself, by redefining `Stream` as
```haskell
type Stream a :: SType -> SType
data Stream a = MkStream a (Stream a)
```

Now that we cannot make infinite streams that simply anymore, we have to shift our thinking from what it means to *build* a stream to what it means to *observe* a stream. In order to achive this, we dualize the stream type by:
1. instead of using a product of components, use a sum of observations
2. instead of demanding the components when building the stream, ask for the decision of what we are going to observe

The former is pretty simple, we define the datatype:
```haskell 
type StreamObservation :: SType -> SType -> SType
data StreamObservation a r
  = Hd (a -> r)
  | Tl (Stream a -> r)
```
As you can see, everything is a strict datatype still so you know we are not cheating you. This datatype now describes what it means to observe a stream:
1. either, you see the head (`Hd`) in which case you may do something with the element you find there, or
2. you will observe the tail (`Tl`) in which case you may do something with the stream that makes up tail.

Now, to restore the interface that we are used to, let's wrap the idea of "give us an observation and we will execute it for you" in a `newtype`:
```haskell
type Stream :: SType -> LType
newtype Stream a = MkStream {unStream :: forall r. StreamObservation a r -> r}
```
As you may have noticed, this returns an `LType`, not an `SType`; the reason is that representationally, a `Stream` is now a function closure, the only primitive lazy datastructure in many strict langauges, just like in our imaginary strict Haskell.
In polarity, which does not have functions but instead has `codata` types, we can write this type out a bit simpler, namely as:
```polarity
codata Stream(a : Type) 
  { Stream(a).head(a : Type): a
  , Stream(a).tail(a : Type): Stream(a)
  }
```
Just like the Haskell version, it offers you two options to observe a stream, but it leaves out the CPS encoding part. This makes it also nicer to create streams with `codef`, which we will see in a minute. In order to do so in haskell, let's define the stream that consists only of ones, that is no matter how far you go (by observing `Tl`), every time you observe head afterwards (`Hd`), you obtain a one.
```haskell
ones :: Stream I
ones = MkStream \case
  Hd k -> k (I# 1#) -- observe the number one that lives in every cell of the Stream
  Tl k -> k ones    -- observe the remainder of the stream
 where
  one = I# 1#
```
in polarity, the definition looks quite similar, except that, instead of using continuation passing style, we can write it in direct style, similarly to how it is done in object oriented programming's class methods:
```polarity
data Nat {Z : Nat, S(n : Nat) : Nat}

codef Ones: Stream(Nat) { -- use codef to define a Stream
  .head(_) => S(Z),       -- when observing the head, return 1
  .tail(_) => Ones        -- otherwise, we observe a stream of ones
}
```

You may have noticed that to model these `codata` types, in contrast to Haskell, Polarity does not need functions. In fact, Polarity has *no* builtin or "first class" function type - the function type can be defined as a `codata` type. For simplicity, here is how type of a function from `Nat` to `Nat` looks: 

```polarity
codata NatToNat { .ap(x: Nat): Nat }
```

It's the type that, if you observe `ap` (the only thing you may observers), requires a `Nat` to be able to give you back a `Nat`. (So just like `newtype NatToNat = NatToNat {ap :: Nat -> Nat}` in Haskell!)

So far so good, now we know how to use the non-dependent portion of polarity and how it corresponds to Haskell.

However, there's another peculiar thing happening: instead of infinitely many numbers, we now only need a single one to construct the stream, so, if we do not duplicate the stream or let the continuation use the result multiple times, there's actually only a single one we can get out of a stream.

We can make that obvious using `-XLinearHaskell` but we have to rewrite some definitions:
```haskell
type StreamObservation :: SType -> SType -> SType
data StreamObservation a r where
  Hd :: (a %1 -> r) %1 -> StreamObservation a r
  Tl :: (Stream a %1 -> r) %1 -> StreamObservation a r

type Stream :: SType -> LType
newtype Stream a where
  MkStream :: (forall r. StreamObservation a r %1 -> r) %1 -> Stream a

unStream :: Stream a %1 -> forall r. StreamObservation a r %1 -> r
unStream (MkStream s) = s
```
Now, if we want to define a stream of some universally quantified `a`, we can write:
```haskell
repeat :: a %1 -> Stream a
repeat x = MkStream \case
  Hd k -> k x
  Tl k -> k (mk x)
```
Now, if we were allowed to use this `a` more than once, the type checker would complain, but since it doesn't, we are good.

So to recap:
- in polarity, making things observable instead of buildable is achieved using `codef` and `codata`. This is equivalent to non-strict semantics, in that we have to pass the continuation that does the observation for us.
- what Haskell does not give you is dependent types, which is why we left them out here - the encoding of dependent types that you have to do in Haskell is much more complex than in polarity
- in Haskell, datatypes can be lazy, which makes it such that in Hakell (in contrast to polarity) values of some lazy (lifted) datatype can represent a computation, not just values. This is not the case in polarity.
