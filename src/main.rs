mod fib;

use std::any::Any;

use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::pasta::Fp;
use plotters::prelude::WHITE;
use plotters::prelude::*;

use halo2_proofs::pasta::{Eq, EqAffine};
use halo2_proofs::plonk::{
    create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Assigned, BatchVerifier, Circuit,
    Column, ConstraintSystem, Error, Fixed, SingleVerifier, TableColumn, VerificationStrategy,
};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge};
use rand_core::OsRng;
use std::time::Instant;

fn main() {
    real_run()
}

fn real_run() {
    let empty_circuit = fib::FibCircuit {
        a: Value::unknown(),
        b: Value::unknown(),
    };
    let circuit = fib::FibCircuit {
        a: Value::known(Fp::one()),
        b: Value::known(Fp::one()),
    };

    let d = Fp::from(3);
    let k = 3;
    let params: Params<EqAffine> = Params::new(k);
    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk.clone(), &empty_circuit).expect("keygen_pk should not fail");
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    // Create a proof
    create_proof(&params, &pk, &[circuit], &[&[&[d]]], OsRng, &mut transcript)
        .expect("proof generation should not fail");
    let proof: Vec<u8> = transcript.finalize();
    println!("{:?}", proof);

    let strategy = SingleVerifier::new(&params);

    let mut transcript2 = Blake2bRead::init(&proof[..]);
    verify_proof(&params, &vk,strategy,&[&[&[d]]],  &mut transcript2).unwrap();
}

fn mock_prover() {
    let circuit = fib::FibCircuit {
        a: Value::known(Fp::one()),
        b: Value::known(Fp::one()),
    };
    let target = Fp::from(5);
    let public_input = vec![target];
    let prover = MockProver::run(3, &circuit, vec![public_input]).unwrap();
    prover.assert_satisfied();
}

fn dev() {
    let root = BitMapBackend::new("fib-layout.png", (1024, 3096)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.titled("Fib Layout", ("sans-serif", 60)).unwrap();

    let circuit = fib::FibCircuit {
        a: Value::known(Fp::one()),
        b: Value::known(Fp::one()),
    };
    halo2_proofs::dev::CircuitLayout::default()
        .show_equality_constraints(true)
        .show_labels(true)
        .render(4, &circuit, &root)
        .unwrap();

    let dot_string = halo2_proofs::dev::circuit_dot_graph(&circuit);
    print!("{}", dot_string);
}
