extern crate amcl_wrapper;
extern crate zmix;
extern crate ursa;
extern crate serde;


use amcl_wrapper::group_elem::GroupElement;
use zmix::signatures::prelude::*;
use zmix::signatures::ps::prelude::*;


use amcl_wrapper::field_elem::{FieldElement, FieldElementVector};

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use amcl_wrapper::extension_field_gt::GT;


//for deserialization
use amcl_wrapper::types::BigNum;
use amcl_wrapper::ECCurve::big::BIG;
use amcl_wrapper::types::FP;
use amcl_wrapper::ECCurve::ecp::ECP;
use amcl_wrapper::group_elem_g1::G1;
use amcl_wrapper::types::FP2;
use amcl_wrapper::ECCurve::ecp2::ECP2;
use amcl_wrapper::group_elem_g2::G2;

// use amcl_wrapper::ECCurve::fp12::FP12;
// use amcl_wrapper::ECCurve::pair::fexp;


use std::mem::transmute;
use std::env;

use std::io::Write;
use std::fs::File;
use std::fmt;



// use std::io::{stdin,stdout,Write};

// use amcl_wrapper::ECCurve::big::BIG;

// use ursa::cl::issuer::Issuer;
// use ursa::cl::prover::Prover;
// use ursa::cl::verifier::Verifier;
// use ursa::cl::*;
// use std::time::{Duration, Instant};

//Create gpk, gmsk




pub fn GSetup (count_msgs: usize, label: &[u8])->(Gpk, Gmsk){
    println!("GSetup Start.........");
    let (gpk, gmsk) = For_GSetup(count_msgs, label);
    // print_type_of(&gpk);
    println!("GSetup Successful!");
    (gpk, gmsk)
}

//Create usk[i] and upk[i] for Gjoin 
pub fn PKIJoin (count_msgs: usize, label: &[u8])->(PublicKey,SecretKey){
    println!("PKIJoin Start.........");
    let (upk_i, usk_i) = keygen(count_msgs, label);
    let msg = FieldElementVector::random(count_msgs);
    let sign_usk_i=Signature::new(msg.as_slice(), &usk_i, &upk_i).unwrap();
    // let check=sign_usk_i.verify(msg.as_slice(),&upk_i).unwrap();
    // println!("usk_i, upk_i pair checks out: {}",check);
    println!("PKIJoin Successful!");
    (upk_i, usk_i)
}

//Need to convert G1 into number so it can be signed
pub fn hashing(s: DefaultHasher,message: amcl_wrapper::group_elem_g1::G1)->u64{
    let mut hasher = s.clone();
    message.hash(&mut hasher);
    hasher.finish()
}
// Need this so tow can be a FieldElementVector
pub fn sign_usk_i(s:DefaultHasher,tow:amcl_wrapper::group_elem_g1::G1,usk_i:SecretKey, upk_i:PublicKey)->Signature{
    let tow_hash=hashing(s.clone(),tow.clone()).to_be_bytes();
    // println!("{:?}", tow_hash);
    let oneMess = FieldElement::from_msg_hash(&tow_hash);
    let mut msg=FieldElementVector::new(0);
    // println!("{:?}", tow_hash % 20 );
    msg.push(oneMess);
    // println!("{:?}", msg);
    Signature::new(msg.as_slice(), &usk_i, &upk_i).unwrap()

}
// Check sign_usk_i signature
pub fn verify_usk_i(signature_usk_i: Signature,s:DefaultHasher,tow:amcl_wrapper::group_elem_g1::G1, upk_i:PublicKey)->bool{

    let tow_hash=hashing(s.clone(),tow.clone()).to_be_bytes();
    let oneMess = FieldElement::from_msg_hash(&tow_hash);
    let mut msg=FieldElementVector::new(0);
    msg.push(oneMess);
    // println!("{:?}", msg);
    let check=signature_usk_i.verify(msg.as_slice(),&upk_i).unwrap();
    check
}

//using interactive sigma protocol, when ski is the only thing given
pub fn test_sigmaProtocol(g:amcl_wrapper::group_elem_g1::G1,y:FieldElement,Y:amcl_wrapper::group_elem_g1::G1)->(){
    //Proofer/USER calculate r and A
    let r = FieldElement::random();
    let A=&g*&r;
    //Proofer send A to Verifer
    //Verifer/GROUP MANAGER Calculate cha
    let cha = FieldElement::random();
    //Verifer send cha to Proofer
    //Proofer calculate rsp
    let rsp=&r-&y*&cha;
    //Proofer send rsp to Verifer
    // Verifer check if A=g^rsp*Y^cha
    let Check=&g*&rsp+&Y*&cha;
    println!("Proof of USER knowing ski: {:?}", A==Check);

}

//Without this the requester cannot sign and if there’s no signature then there’s nothing to verify
pub fn GJoin (i: usize, gpk: Gpk,gmsk: Gmsk, upk_i:PublicKey ,usk_i:SecretKey)->((usize,amcl_wrapper::group_elem_g1::G1,Signature,amcl_wrapper::group_elem_g2::G2,DefaultHasher), (amcl_wrapper::field_elem::FieldElement, (amcl_wrapper::group_elem_g1::G1, amcl_wrapper::group_elem_g1::G1), String)){
    println!("GJoin Start.........");
    //USER generates a secret key,τ, τ_tidle, η and send τ, τ_tidle and η
    println!("USER create ski, τ, τ_tidle and η and send τ, τ_tidle and η");
    let ski= FieldElement::random();
    let tow=&gpk.g * &ski;
    let tow_tilde= &gpk.Y_tilde * &ski;
    let mut hash_saved = DefaultHasher::new();
    let n =sign_usk_i(hash_saved.clone(), tow.clone(), usk_i.clone(), upk_i.clone());
    // let m =sign_usk_i(s.clone(), tow.clone(), usk_i.clone(), upk_i.clone());
    // let check1=verify_usk_i(n.clone(),s.clone(), tow.clone(),upk_i.clone());
    // let check2=verify_usk_i(m.clone(),s.clone(), tow.clone(),upk_i.clone());
    // println!("{:?}",check1);
    // println!("{:?}",check2);
    

    //GROUP MANAGER tests e(τ, Y_tilde) =e(g, τ_tilde)
    let res = GT::ate_pairing(&tow, &gpk.Y_tilde);
    let res2 = GT::ate_pairing(&gpk.g, &tow_tilde);
    println!("GROUP MANAGER tests e(τ, Y_tilde) =e(g, τ_tilde): {:?}", res==res2);
    // println!("{:?}", res==res2);


    println!("USER Start Proof of knowledge of ski");
    //User start proof of knowledge for ski
    // let pk=(&tow, &gpk.Y_tilde);
    // test_PoK_multiple_sigs(pk,ski);
    test_sigmaProtocol(gpk.g.clone(),ski.clone(),tow.clone());
    

    println!("Group Manager Generates u, σ");
    //Group MANAGER u, σ←(σ1,σ2)←(gu,(gx·(τ)y)u) 
    let u= FieldElement::random();
    let sigma1=&gpk.g * &u;
    //(g^x·(τ)^y)^u=g^x^u·(τ)^y^u IS this true?????
    let sigma2=&gpk.g * &gmsk.x * &u + &tow * &gmsk.y * &u;
    let sigma=(sigma1.clone(),sigma2.clone());


    println!("Group Manager Stores i,τ,η,τ_tilde and hash");
    //Group Manager Store (i,τ,η,τ_tilde) need to add s for hasher
    let secret_register=(i,tow,n,tow_tilde,hash_saved);

    println!("USER Stores ski,σ,e(σ1,Y_tilde)");
    //User Store (ski,σ,e(σ1,Y_tilde))
    let gsk_i=(ski,sigma,GT::ate_pairing(&sigma1,&gpk.Y_tilde).to_hex());

    println!("GJoin Successful!");

    (secret_register,gsk_i)

}



pub fn GJoin2 (i: usize, gpk: Gpk,gmsk: Gmsk, upk_i:PublicKey ,usk_i:SecretKey)->((usize,String,Signature,String,DefaultHasher), (String, (String, String), String)){
    println!("GJoin Start.........");
    //USER generates a secret key,τ, τ_tidle, η and send τ, τ_tidle and η
    println!("USER create ski, τ, τ_tidle and η and send τ, τ_tidle and η");
    let ski= FieldElement::random();
    let tow=&gpk.g * &ski;
    let tow_tilde= &gpk.Y_tilde * &ski;
    let mut hash_saved = DefaultHasher::new();
    let n =sign_usk_i(hash_saved.clone(), tow.clone(), usk_i.clone(), upk_i.clone());
    // let m =sign_usk_i(s.clone(), tow.clone(), usk_i.clone(), upk_i.clone());
    // let check1=verify_usk_i(n.clone(),s.clone(), tow.clone(),upk_i.clone());
    // let check2=verify_usk_i(m.clone(),s.clone(), tow.clone(),upk_i.clone());
    // println!("{:?}",check1);
    // println!("{:?}",check2);
    

    //GROUP MANAGER tests e(τ, Y_tilde) =e(g, τ_tilde)
    let res = GT::ate_pairing(&tow, &gpk.Y_tilde);
    let res2 = GT::ate_pairing(&gpk.g, &tow_tilde);
    println!("GROUP MANAGER tests e(τ, Y_tilde) =e(g, τ_tilde): {:?}", res==res2);
    // println!("{:?}", res==res2);


    println!("USER Start Proof of knowledge of ski");
    //User start proof of knowledge for ski
    // let pk=(&tow, &gpk.Y_tilde);
    // test_PoK_multiple_sigs(pk,ski);
    test_sigmaProtocol(gpk.g.clone(),ski.clone(),tow.clone());
    

    println!("Group Manager Generates u, σ");
    //Group MANAGER u, σ←(σ1,σ2)←(gu,(gx·(τ)y)u) 
    let u= FieldElement::random();
    let sigma1=&gpk.g * &u;
    //(g^x·(τ)^y)^u=g^x^u·(τ)^y^u IS this true?????
    let sigma2=&gpk.g * &gmsk.x * &u + &tow * &gmsk.y * &u;
    let sigma=(sigma1.to_hex(),sigma2.to_hex());


    println!("Group Manager Stores i,τ,η,τ_tilde and hash");
    //Group Manager Store (i,τ,η,τ_tilde) need to add s for hasher
    let secret_register=(i,tow.to_hex(),n,tow_tilde.to_hex(),hash_saved);

    println!("USER Stores ski,σ,e(σ1,Y_tilde)");
    //User Store (ski,σ,e(σ1,Y_tilde))
    let gsk_i=(ski.to_hex(),sigma,GT::ate_pairing(&sigma1,&gpk.Y_tilde).to_hex());

    println!("GJoin Successful!");

    (secret_register,gsk_i)

}



// BackupGJoin
// pub fn GJoin (i: usize, gpk: Gpk,gmsk: Gmsk, upk_i:PublicKey ,usk_i:SecretKey)->((usize,amcl_wrapper::group_elem_g1::G1,Signature,amcl_wrapper::group_elem_g2::G2,DefaultHasher), (amcl_wrapper::field_elem::FieldElement, (amcl_wrapper::group_elem_g1::G1, amcl_wrapper::group_elem_g1::G1), amcl_wrapper::extension_field_gt::GT)){
//     println!("GJoin Start.........");
//     //USER generates a secret key,τ, τ_tidle, η and send τ, τ_tidle and η
//     println!("USER create ski, τ, τ_tidle and η and send τ, τ_tidle and η");
//     let ski= FieldElement::random();
//     let tow=&gpk.g * &ski;
//     let tow_tilde= &gpk.Y_tilde * &ski;
//     let mut hash_saved = DefaultHasher::new();
//     let n =sign_usk_i(hash_saved.clone(), tow.clone(), usk_i.clone(), upk_i.clone());
//     // let m =sign_usk_i(s.clone(), tow.clone(), usk_i.clone(), upk_i.clone());
//     // let check1=verify_usk_i(n.clone(),s.clone(), tow.clone(),upk_i.clone());
//     // let check2=verify_usk_i(m.clone(),s.clone(), tow.clone(),upk_i.clone());
//     // println!("{:?}",check1);
//     // println!("{:?}",check2);
    

//     //GROUP MANAGER tests e(τ, Y_tilde) =e(g, τ_tilde)
//     let res = GT::ate_pairing(&tow, &gpk.Y_tilde);
//     let res2 = GT::ate_pairing(&gpk.g, &tow_tilde);
//     println!("GROUP MANAGER tests e(τ, Y_tilde) =e(g, τ_tilde): {:?}", res==res2);
//     // println!("{:?}", res==res2);


//     println!("USER Start Proof of knowledge of ski");
//     //User start proof of knowledge for ski
//     // let pk=(&tow, &gpk.Y_tilde);
//     // test_PoK_multiple_sigs(pk,ski);
//     test_sigmaProtocol(gpk.g.clone(),ski.clone(),tow.clone());
    

//     println!("Group Manager Generates u, σ");
//     //Group MANAGER u, σ←(σ1,σ2)←(gu,(gx·(τ)y)u) 
//     let u= FieldElement::random();
//     let sigma1=&gpk.g * &u;
//     //(g^x·(τ)^y)^u=g^x^u·(τ)^y^u IS this true?????
//     let sigma2=&gpk.g * &gmsk.x * &u + &tow * &gmsk.y * &u;
//     let sigma=(sigma1.clone(),sigma2.clone());


//     println!("Group Manager Stores i,τ,η,τ_tilde and hash");
//     //Group Manager Store (i,τ,η,τ_tilde) need to add s for hasher
//     let secret_register=(i,tow,n,tow_tilde,hash_saved);

//     println!("USER Stores ski,σ,e(σ1,Y_tilde)");
//     //User Store (ski,σ,e(σ1,Y_tilde))
//     let gsk_i=(ski,sigma,GT::ate_pairing(&sigma1,&gpk.Y_tilde));

//     println!("GJoin Successful!");

//     (secret_register,gsk_i)

// }


//Hash tuple of messy G1,GT,str
pub fn H1(s:DefaultHasher,message:(amcl_wrapper::group_elem_g1::G1,
    amcl_wrapper::group_elem_g1::G1,
    amcl_wrapper::extension_field_gt::GT,String))->u64{
    let mut hasher = s.clone();
    message.hash(&mut hasher);
    hasher.finish()
}

// Requester sign message with ski[i] and outputs signature and message
pub fn GSign(gsk_i:(amcl_wrapper::field_elem::FieldElement, 
    (amcl_wrapper::group_elem_g1::G1, 
        amcl_wrapper::group_elem_g1::G1),
    String),msg:String)->(
    (amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::field_elem::FieldElement, 
    amcl_wrapper::field_elem::FieldElement), 
    DefaultHasher,String){
    // println!("GSign Start.........");
    // let msg="test_message";
    let ski=gsk_i.0;
    let sigma1=gsk_i.1.0;
    let sigma2=gsk_i.1.1;
    let e=GT::from_hex(gsk_i.2).unwrap();

    //USER Create t and  computing  (σ′1,σ′2)←(σt1,σt2)

    ////////sen needs to be random
    let t = FieldElement::random();
    // let t=FieldElement::one();
    ////////sen needs to be random

    let sigma1_dash=sigma1 * &t;
    let sigma2_dash=sigma2 * &t;

    //USER create a  signature  of  knowledge  ofski.

    ////////sen needs to be random
    let k = FieldElement::random();
    // let k = FieldElement::one();
    ////////sen needs to be random

    // e(σ′1, Y_tilde)^k←e(σ1, Y_tilde)^k·t
    let e_tok_tot=e.pow(&k).pow(&t);

    //Please note code need to convert (σ′1,σ′2,e(σ1, Y_tilde)^k·t,m) to a hash u8 so this tuple can be converted into Fieldelement form using from_msg_hash
    let mut hash_saved = DefaultHasher::new();
    let number = H1(hash_saved.clone(),(sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg.clone())).to_be_bytes();
    // let number = let bytes: [u8; 4] = unsafe { transmute(H1((sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg)).to_be()) };
    // let number2 = H1(ss.clone(),(sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg)).to_be_bytes();
    // println!("{:?}", number);

    //c needs to be a fieldElement
    let c = FieldElement::from_msg_hash(&number);
    // //make sure hash consistent
    // let c2 = FieldElement::from_msg_hash(&number);
    // println!("{:?}", c);
    // println!("{:?}", c2);

    // USER Compute s←k+c·ski
    let s = &k + &c * &ski;

    //Output outputs (σ′1,σ′2,c,s) and m
    let mu=(sigma1_dash,sigma2_dash,c,s);
    // println!("GSign Successful!");

    (mu,hash_saved.clone(), msg)

}


pub fn GSign2(gpk: Gpk,gsk_i:(amcl_wrapper::field_elem::FieldElement, 
    (amcl_wrapper::group_elem_g1::G1, 
        amcl_wrapper::group_elem_g1::G1),
    String),msg:String)->(
    (String, 
    String, 
    String, 
    String), 
    DefaultHasher,String){
    // println!("GSign Start.........");
    // let msg="test_message";
    let ski=gsk_i.0;
    let sigma1=gsk_i.1.0;
    let sigma2=gsk_i.1.1;
    let e=GT::from_hex(gsk_i.2).unwrap();

    //USER Create t and  computing  (σ′1,σ′2)←(σt1,σt2)

    ////////sen needs to be random
    let t = FieldElement::random();
    // let t=FieldElement::one();
    ////////sen needs to be random

    let sigma1_dash=sigma1 * &t;
    let sigma2_dash=sigma2 * &t;

    //USER create a  signature  of  knowledge  ofski.

    ////////sen needs to be random
    let k = FieldElement::random();
    // let k = FieldElement::one();
    ////////sen needs to be random

    // e(σ′1, Y_tilde)^k←e(σ1, Y_tilde)^k·t
    let e_tok_tot=e.pow(&k).pow(&t);

    //Please note code need to convert (σ′1,σ′2,e(σ1, Y_tilde)^k·t,m) to a hash u8 so this tuple can be converted into Fieldelement form using from_msg_hash
    let mut hash_saved = DefaultHasher::new();
    let number = H1(hash_saved.clone(),(sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg.clone())).to_be_bytes();
    // let number = let bytes: [u8; 4] = unsafe { transmute(H1((sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg)).to_be()) };
    // let number2 = H1(ss.clone(),(sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg)).to_be_bytes();
    // println!("{:?}", number);

    //c needs to be a fieldElement
    let c = FieldElement::from_msg_hash(&number);
    // //make sure hash consistent
    // let c2 = FieldElement::from_msg_hash(&number);
    // println!("{:?}", c);
    // println!("{:?}", c2);

    // println!("e_tok_tot:{:?}\n", &e_tok_tot);

    // println!("e:{:?}\n", &e);
    // println!("ski:{:?}\n", &ski);
    // println!("c:{:?}\n", &c);
    // println!("k:{:?}\n", &k);
    


    // USER Compute s←k+c·ski
    let s = &k + &c * &ski;

    // println!("s:{:?}\n", &s);

    //Output outputs (σ′1,σ′2,c,s) and m
    let mu=(sigma1_dash.to_hex(),sigma2_dash.to_hex(),c.to_hex(),s.to_hex());
    // println!("ski:{:?}\n", &ski);
    // println!("c:{:?}\n", &c);
    // println!("k:{:?}\n", &k);
    println!("msg:{:?}\n", &msg);


    /////////////testverify
    // let mu_true = (G1::from_hex(mu.clone().0).unwrap(),G1::from_hex(mu.clone().1).unwrap(),FieldElement::from_hex(mu.clone().2).unwrap(),FieldElement::from_hex(mu.clone().3).unwrap());
    // let verified_signature_1=GVerify(gpk.clone(),mu_true.clone(),hash_saved.clone(), msg.clone());

    // println!("verified_signature_2 is: {:?} \n", verified_signature_1);
    ////////////


    (mu,hash_saved.clone(), msg)

}





//backup GSign
// pub fn GSign(gsk_i:(amcl_wrapper::field_elem::FieldElement, 
//     (amcl_wrapper::group_elem_g1::G1, 
//         amcl_wrapper::group_elem_g1::G1),
//     amcl_wrapper::extension_field_gt::GT),msg:&'static  str)->(
//     (amcl_wrapper::group_elem_g1::G1, 
//     amcl_wrapper::group_elem_g1::G1, 
//     amcl_wrapper::field_elem::FieldElement, 
//     amcl_wrapper::field_elem::FieldElement), 
//     DefaultHasher,&'static  str){
//     // println!("GSign Start.........");
//     // let msg="test_message";
//     let ski=gsk_i.0;
//     let sigma1=gsk_i.1.0;
//     let sigma2=gsk_i.1.1;
//     let e=gsk_i.2;

//     //USER Create t and  computing  (σ′1,σ′2)←(σt1,σt2)
//     let t = FieldElement::random();
//     let sigma1_dash=sigma1 * &t;
//     let sigma2_dash=sigma2 * &t;

//     //USER create a  signature  of  knowledge  ofski.
//     let k = FieldElement::random();
//     // e(σ′1, Y_tilde)^k←e(σ1, Y_tilde)^k·t
//     let e_tok_tot=e.pow(&k).pow(&t);

//     //Please note code need to convert (σ′1,σ′2,e(σ1, Y_tilde)^k·t,m) to a hash u8 so this tuple can be converted into Fieldelement form using from_msg_hash
//     let mut hash_saved = DefaultHasher::new();
//     let number = H1(hash_saved.clone(),(sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg)).to_be_bytes();
//     // let number = let bytes: [u8; 4] = unsafe { transmute(H1((sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg)).to_be()) };
//     // let number2 = H1(ss.clone(),(sigma1_dash.clone(),sigma2_dash.clone(),e_tok_tot.clone(),msg)).to_be_bytes();
//     // println!("{:?}", number);

//     //c needs to be a fieldElement
//     let c = FieldElement::from_msg_hash(&number);
//     // //make sure hash consistent
//     // let c2 = FieldElement::from_msg_hash(&number);
//     // println!("{:?}", c);
//     // println!("{:?}", c2);

//     // USER Compute s←k+c·ski
//     let s = &k + &c * &ski;

//     //Output outputs (σ′1,σ′2,c,s) and m
//     let mu=(sigma1_dash,sigma2_dash,c,s);
//     // println!("GSign Successful!");

//     (mu,hash_saved.clone(), msg)

// }

//Verify Requester Group ID
pub fn GVerify2(gpk: Gpk,mu:(amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::field_elem::FieldElement, 
    amcl_wrapper::field_elem::FieldElement), 
    hash_for_tuple:DefaultHasher,msg:String)->bool{

    // println!("GVerify Start.........");
    let sigma1_dash=mu.0;
    let sigma2_dash=mu.1;
    let c=mu.2;
    let c1=c.clone();
    let s=mu.3;

    // Verifier computes R←(e(σ1^-1, X_tilde)·e(σ2, g_tilde))−c·e(σs1, Y_tilde) 
    // let b = &-c; //also works, but slo wer?
    let b =&c.negation();
    //Assuming (e(g1,g2)*e(h1,h2))^-c ==e(g1^-c,g2)*e(h1^-c,h2)
    let R =GT::ate_multi_pairing(vec![(&(-&sigma1_dash).scalar_mul_variable_time(b),&gpk.X_tilde),
        (&sigma2_dash.scalar_mul_variable_time(b),&gpk.g_tilde),
        (&sigma1_dash.scalar_mul_variable_time(&s),&gpk.Y_tilde)]);
    let number = H1(hash_for_tuple.clone(),(sigma1_dash.clone(),sigma2_dash.clone(),R.clone(),msg.clone())).to_be_bytes();
    let c2 = FieldElement::from_msg_hash(&number);
    // Verify that c=H(σ1,σ2,R,m);
    // println!("Does this Verify: {:?}", c1==c2);
    // println!("message: {:?}", msg);


    // println!("number: {:?}", number);
    // println!("R: {:?}", R);


    // println!("b: {:?}", b);
    // println!("sigma2_dash: {:?}", sigma2_dash.scalar_mul_variable_time(&c));
    // println!("msg: {:?}", msg);
    // println!("GVerify Successful!");
    c1==c2


    // a=e(σ1^-1, X_tilde)·e(σ2, g_tilde))^−c

    // let a = GT::ate_2_pairing(&(-&sigma1_dash),&gpk.X_tilde,&sigma2_dash,&gpk.g_tilde).pow(&-c);
    // // b=e(σ1^s, Y_tilde)=e(σ1, Y_tilde)^s;
    // // let b = GT::ate_pairing(&sigma1_dash,&gpk.Y_tilde).pow(&s);
    // let b=GT::ate_pairing(&sigma1_dash.scalar_mul_variable_time(&s),&gpk.Y_tilde);
    // // R=a·b
    // // let R = a*b;

    // let b = &-c;


    // let e_vector = Vec::new();

    // let sig1_inverse=-&sigma1_dash;
    // e_vector.push((sig1_inverse,gpk.X_tilde));

    // e_vector.push((sigma2_dash,gpk.g_tilde));

    // let sig1_to_s=sigma1_dash.scalar_mul_variable_time(&s);
    // e_vector.push((sig1_to_s,gpk.Y_tilde));


    // let a1 = GT::ate_2_pairing(&(-&sigma1_dash),&gpk.X_tilde,&sigma2_dash,&gpk.g_tilde).pow(&s);
    // let a2 = GT::ate_2_pairing(&(-&sigma1_dash).scalar_mul_variable_time(&s),&gpk.X_tilde,&sigma2_dash.scalar_mul_variable_time(&s),&gpk.g_tilde);
    // println!("{:?}", a1==a2);

    //e(g1,g2)^s=e(g1^s,g2);
    // let b2=GT::ate_pairing(&sigma1_dash.scalar_mul_variable_time(&s),&gpk.Y_tilde);
    // println!("{:?}", b==b2);



    // let test1=GT::ate_pairing(&(-&sigma1_dash),&gpk.Y_tilde);
    // let test1_1=GT::ate_pairing(&sigma1_dash,&gpk.Y_tilde);
    // let test2=GT::ate_pairing(&sigma1_dash,&gpk.Y_tilde).inverse();

    // println!("{:?}", test1==test2);
    // println!("{:?}", test1_1==test2);

    // let r=ate_pairing();

}


//Verify Requester Group ID
pub fn GVerify(gpk: Gpk,mu:(amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::field_elem::FieldElement, 
    amcl_wrapper::field_elem::FieldElement), 
    hash_for_tuple:DefaultHasher,msg:String)->bool{

    // println!("GVerify Start.........");
    let sigma1_dash=mu.0;
    let sigma2_dash=mu.1;
    let c=mu.2;
    let c1=c.clone();
    let s=mu.3;

    // Verifier computes R←(e(σ1^-1, X_tilde)·e(σ2, g_tilde))−c·e(σs1, Y_tilde) 
    // let b = &-c; //also works, but slo wer?
    let b =&c.negation();
    //Assuming (e(g1,g2)*e(h1,h2))^-c ==e(g1^-c,g2)*e(h1^-c,h2)
    let R =GT::ate_multi_pairing(vec![(&(-&sigma1_dash).scalar_mul_variable_time(b),&gpk.X_tilde),
        (&sigma2_dash.scalar_mul_variable_time(b),&gpk.g_tilde),
        (&sigma1_dash.scalar_mul_variable_time(&s),&gpk.Y_tilde)]);
    let number = H1(hash_for_tuple.clone(),(sigma1_dash.clone(),sigma2_dash.clone(),R.clone(),msg.clone())).to_be_bytes();
    let c2 = FieldElement::from_msg_hash(&number);
    // Verify that c=H(σ1,σ2,R,m);
    // println!("Does this Verify: {:?}", c1==c2);
    // println!("message: {:?}", msg);


    // println!("sigma1_dash: {:?}", sigma1_dash);
    // println!("sigma2_dash: {:?}", sigma2_dash);
    // println!("c: {:?}", c);
    // println!("c1: {:?}", c1);
    // println!("c2: {:?}", c2);
    // println!("GVerify Successful!");
    c1==c2


    // a=e(σ1^-1, X_tilde)·e(σ2, g_tilde))^−c

    // let a = GT::ate_2_pairing(&(-&sigma1_dash),&gpk.X_tilde,&sigma2_dash,&gpk.g_tilde).pow(&-c);
    // // b=e(σ1^s, Y_tilde)=e(σ1, Y_tilde)^s;
    // // let b = GT::ate_pairing(&sigma1_dash,&gpk.Y_tilde).pow(&s);
    // let b=GT::ate_pairing(&sigma1_dash.scalar_mul_variable_time(&s),&gpk.Y_tilde);
    // // R=a·b
    // // let R = a*b;

    // let b = &-c;


    // let e_vector = Vec::new();

    // let sig1_inverse=-&sigma1_dash;
    // e_vector.push((sig1_inverse,gpk.X_tilde));

    // e_vector.push((sigma2_dash,gpk.g_tilde));

    // let sig1_to_s=sigma1_dash.scalar_mul_variable_time(&s);
    // e_vector.push((sig1_to_s,gpk.Y_tilde));


    // let a1 = GT::ate_2_pairing(&(-&sigma1_dash),&gpk.X_tilde,&sigma2_dash,&gpk.g_tilde).pow(&s);
    // let a2 = GT::ate_2_pairing(&(-&sigma1_dash).scalar_mul_variable_time(&s),&gpk.X_tilde,&sigma2_dash.scalar_mul_variable_time(&s),&gpk.g_tilde);
    // println!("{:?}", a1==a2);

    //e(g1,g2)^s=e(g1^s,g2);
    // let b2=GT::ate_pairing(&sigma1_dash.scalar_mul_variable_time(&s),&gpk.Y_tilde);
    // println!("{:?}", b==b2);



    // let test1=GT::ate_pairing(&(-&sigma1_dash),&gpk.Y_tilde);
    // let test1_1=GT::ate_pairing(&sigma1_dash,&gpk.Y_tilde);
    // let test2=GT::ate_pairing(&sigma1_dash,&gpk.Y_tilde).inverse();

    // println!("{:?}", test1==test2);
    // println!("{:?}", test1_1==test2);

    // let r=ate_pairing();

}


//Used as last resort to find identity, Note need to know gpk since need g.tilde and X_tilde
pub fn GOpen(gpk: Gpk,gmsk_array: Vec<(usize,amcl_wrapper::group_elem_g1::G1,Signature,amcl_wrapper::group_elem_g2::G2,DefaultHasher)>, mu:(amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::field_elem::FieldElement, 
    amcl_wrapper::field_elem::FieldElement), 
    hash_for_tuple:DefaultHasher,msg:&'static  str)->(){

    let sigma1_dash=mu.0;
    let sigma2_dash=mu.1;
    let c=mu.2;
    let s=mu.3;
    // let mut true_tow_tilde: amcl_wrapper::group_elem_g2::G2;
    // let mut true_identity: (usize,amcl_wrapper::group_elem_g1::G1,Signature);

    //loop to find the user
    for gmsk in gmsk_array{
        let idenity_id= gmsk.0;
        let tow = gmsk.1;
        let n = gmsk.2;
        let tow_tilde = gmsk.3;
        let hash_saved = gmsk.4;

        //check e(σ2, g_tilde)·e(σ1, X_tilde)^−1=e(σ1, τ_tilde)
        if GT::ate_2_pairing(&sigma2_dash,&gpk.g_tilde,&(-&sigma1_dash),&gpk.X_tilde)==GT::ate_pairing(&sigma1_dash,&tow_tilde){
            println!("The identity is User {:?}", idenity_id);
            let true_tow_tilde=tow_tilde;
            let true_identity=(idenity_id,tow,n);

            //Proof of knowledge of τ_tilde
            //GM informs all to chanellege it's knowledge of τ_tilde
            //Verifer generates r and A
            let r = FieldElement::random();
            let cha = &gpk.g*&r;
            //Verifer sends cha to Proofer/GM, GM calculates rsp=e(A,τ_tilde)
            let rsp = GT::ate_pairing(&cha,&true_tow_tilde);
            //GM sends rsp to Verifer
            //Verifer calculates e(τ,Y_tilde)^r and check if rsp=e(τ,Y_tilde)^r
            println!("Proof of knowledge of τ_tilde {:?}", rsp==GT::ate_pairing(&true_identity.1,&gpk.Y_tilde).pow(&r));


        }
    }

}




// fn input_user()->(std::string::String){
//     use std::io::{stdin,stdout,Write};
//     let mut s=String::new();
//     println!("Please enter some text: ");
//     let _=stdout().flush();
//     stdin().read_line(&mut s).expect("Did not enter a correct string");
//     s
// }

fn input_user_line()->(std::string::String){
    let args: Vec<String> = env::args().collect();
    let mut string: String = args[2].clone();
    // std::io::stdin().read_line(&mut string);
    string
}



fn deserialize_G1(string_21 : String)->(amcl_wrapper::group_elem_g1::G1){
    //deserialize G1

    // let string_21="G1{value:ECP:[FP:[BIG:[0F14C406809B10B4C15144BB87CF0BD1017273A17FCC10107F3C8DFDA475195AD8150539A58030706C56BFBE308DC589]],FP:[BIG:[00224B1552B3263C88E87B320DD5F8A42BEE25680CDED3C068495C70E64CC854AA214BECD05CFBDAAA6FB94ABC346D31]],FP:[BIG:[13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84]]]}";

    let vec3: Vec<&str> = string_21.split(r#"FP:[BIG:["#).collect();
    // println!("Your Big is: {:?} ", vec3);
    let string_211=vec3[1].replace("]],", "");

    let string_212=vec3[2].replace("]],", "");

    let string_213=vec3[3].replace("]]]}", "");

    //Big:

    let big_num_211=BIG::from_hex(string_211.to_string());

    let big_num_212=BIG::from_hex(string_212.to_string());

    let big_num_213=BIG::from_hex(string_213.to_string());

    // println!("Your Big is: {:?} ", big_num_211);

    // println!("Your Big is: {:?} ", big_num_212);

    // println!("Your Big is: {:?} ", big_num_213);


    //FP
    // let fp_211=FP::new_big(&big_num_211);

    // let fp_212=FP::new_big(&big_num_212);

    // let fp_213=FP::new_big(&big_num_213);

    let fp_211=FP { x: big_num_211, xes:0 };

    let fp_212=FP { x: big_num_212, xes:0 };

    let fp_213=FP { x: big_num_213, xes:0 };


    // println!("Your FP is: {:?} ", fp_211);

    // println!("Your FP is: {:?} ", fp_212);

    // println!("Your FP is: {:?} ", fp_213);

    //ECP
    let mut ecp_21=ECP::new();
    ecp_21.setpx(fp_211);
    ecp_21.setpy(fp_212);
    ecp_21.setpz(fp_213);


    // println!("Your ECP is: {:?} ", ecp_21);


    //G1
    let G1_21_element = G1::from(ecp_21);

    // println!("Your G1 is: {:?} ", G1_21_element);
    G1_21_element

}

fn deserialize_G1_2(string_21 : String)->(amcl_wrapper::group_elem_g1::G1){
    //deserialize G1

    // let string_21="G1{value:ECP:[FP:[BIG:[0F14C406809B10B4C15144BB87CF0BD1017273A17FCC10107F3C8DFDA475195AD8150539A58030706C56BFBE308DC589]],FP:[BIG:[00224B1552B3263C88E87B320DD5F8A42BEE25680CDED3C068495C70E64CC854AA214BECD05CFBDAAA6FB94ABC346D31]],FP:[BIG:[13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84]]]}";

    let vec3: Vec<&str> = string_21.split(r#"FP: [ BIG: [ "#).collect();
    // println!("Your Big is: {:?} ", vec3);
    let string_211=vec3[1].replace(" ] ], ", "");

    let string_212=vec3[2].replace(" ] ], ", "");

    let string_213=vec3[3].replace(" ] ] ] }", "");

    //Big:

    let big_num_211=BIG::from_hex(string_211.to_string());

    let big_num_212=BIG::from_hex(string_212.to_string());

    let big_num_213=BIG::from_hex(string_213.to_string());

    // println!("Your Big is: {:?} ", big_num_211);

    // println!("Your Big is: {:?} ", big_num_212);

    // println!("Your Big is: {:?} ", big_num_213);


    //FP
    // let fp_211=FP::new_big(&big_num_211);

    // let fp_212=FP::new_big(&big_num_212);

    // let fp_213=FP::new_big(&big_num_213);

    let fp_211=FP { x: big_num_211, xes:0 };

    let fp_212=FP { x: big_num_212, xes:0 };

    let fp_213=FP { x: big_num_213, xes:0 };


    // println!("Your FP is: {:?} ", fp_211);

    // println!("Your FP is: {:?} ", fp_212);

    // println!("Your FP is: {:?} ", fp_213);

    //ECP
    let mut ecp_21=ECP::new();
    ecp_21.setpx(fp_211);
    ecp_21.setpy(fp_212);
    ecp_21.setpz(fp_213);


    // println!("Your ECP is: {:?} ", ecp_21);


    //G1
    let G1_21_element = G1::from(ecp_21);

    // println!("Your G1 is: {:?} ", G1_21_element);
    G1_21_element

}


fn deserialize_G2(string_21 : String)->(amcl_wrapper::group_elem_g2::G2){
    //deserialize G1

    let g_tilde="G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 04BC772E1ED2D6B1BA5EE7CD66CCD9D16F5FA64A9876657FD9FB996EB60E096EF41C7BC96AA71E85C12478931EDE5C19 ] ], FP: [ BIG: [ 15D71FCB9F9BAABCCA2B622B4C2FF7658626375728546DDF32F3E4F91DDDE9DD149B40C1CA942A2AABDFC25C3443D767 ] ] ], FP2: [ FP: [ BIG: [ 0CC195A47A6A4F0329F4304C232C5C49121EFD8F53267B45BE721A7898EA863A1C4F2155A5C0ED4A31F33D126D15655D ] ], FP: [ BIG: [ 0F6BE700BDE004D83FC202E5DA8BFED0D9C04EDC76853FDEA0D1CA1B74BB11D5D69F261679ED69850BA6441D0B0F2B76 ] ] ], FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ], FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] }";
    
    // println!("Old: {:?}", string_21);

    let vec3: Vec<&str> = string_21.split(r#"FP: [ BIG: [ "#).collect();

    

    let string_1=vec3[1].replace(" ] ], ", "");

    let string_2=vec3[2].replace(" ] ] ], FP2: [ ", "");

    let string_3=vec3[3].replace(" ] ], ", "");

    let string_4=vec3[4].replace(" ] ] ], FP2: [ ", "");

    let string_5=vec3[5].replace(" ] ], ", "");

    let string_6=vec3[6].replace(" ] ] ] ] }", "");

    // println!("{:?}", string_1);
    // println!("{:?}", string_2);
    // println!("{:?}", string_3);
    // println!("{:?}", string_4);
    // println!("{:?}", string_5);
    // println!("{:?}", string_6);



    //Big:

    let big_num_1=BIG::from_hex(string_1.to_string());

    let big_num_2=BIG::from_hex(string_2.to_string());

    let big_num_3=BIG::from_hex(string_3.to_string());

    let big_num_4=BIG::from_hex(string_4.to_string());

    let big_num_5=BIG::from_hex(string_5.to_string());

    let big_num_6=BIG::from_hex(string_6.to_string());

    // println!("Your Big is: {:?} ", big_num_211);

    // println!("Your Big is: {:?} ", big_num_212);

    // println!("Your Big is: {:?} ", big_num_213);


    //FP
    // let fp_211=FP2::new_big(&big_num_211);

    // let fp_212=FP2::new_big(&big_num_212);

    // let fp_213=FP2::new_big(&big_num_213);

    let fp_1=FP { x: big_num_1, xes:0 };

    let fp_2=FP { x: big_num_2, xes:0 };

    let fp_3=FP { x: big_num_3, xes:0 };

    let fp_4=FP { x: big_num_4, xes:0 };

    let fp_5=FP { x: big_num_5, xes:0 };

    let fp_6=FP { x: big_num_6, xes:0 };


    //FP2
    let fp2_1=FP2::new_fps(&fp_1,&fp_2);

    let fp2_2=FP2::new_fps(&fp_3,&fp_4);

    let fp2_3=FP2::new_fps(&fp_5,&fp_6);


    // println!("{:?}", fp2_1);
    // println!("{:?}", fp2_2);
    // println!("{:?}", fp2_3);


    // println!("Your FP is: {:?} ", fp_211);

    // println!("Your FP is: {:?} ", fp_212);

    // println!("Your FP is: {:?} ", fp_213);

    //ECP
    let mut ecp_21=ECP2::new();
    ecp_21.setpx(fp2_1);
    ecp_21.setpy(fp2_2);
    ecp_21.setpz(fp2_3);


    // println!("Your ECP is: {:?} ", ecp_21);


    //G2
    let G2_21_element = G2::from(ecp_21);

    // println!("Your G2 is: {:?} ", G2_21_element);
    G2_21_element

}

fn deserialize_GroupPublicKey(string_gpk : String)->(zmix::signatures::ps::keys::Gpk){
    //deserialize gpk
    // println!("{:?}",string_gpk);

    let vec_g_1: Vec<&str> = string_gpk.split(r#"g: "#).collect();
    let vec_g_2: Vec<&str> = vec_g_1[1].split(r#", g_tilde: "#).collect();

    let g_string=vec_g_2[0].replace(r#"] ] ] } "#,r#"] ] ] }"#);

    let vec_g_tilde_1: Vec<&str> = vec_g_2[1].split(r#", X_tilde: "#).collect();
    let g_tilde_string=vec_g_tilde_1[0].replace(r#"] ] ] } "#,r#"] ] ] }"#);

    let vec_X_tilde_1: Vec<&str> = vec_g_tilde_1[1].split(r#", Y_tilde: "#).collect();
    let X_tilde_string=vec_X_tilde_1[0].replace(r#"] ] ] } "#,r#"] ] ] }"#);

    let Y_tilde_string = vec_X_tilde_1[1].replace(r#"}}"#,r#"}"#).replace(r#"] ] ] } } "#,r#"] ] ] }"#);
    
    //gpk expected form
    //let g_string="G1 { value: ECP: [ FP: [ BIG: [ 176827B765AC12FFF95F1191D8C8AA2EC805EAF1BCE7710E2F6ED3AACAF125E1FAC1E6AC9C32B8925D77E26A5C9CAEEC ] ], FP: [ BIG: [ 0814B11A08E2E3014E958D84B74D0DAB030541EC7794EAC69D052CDA6926C9A2C53B63A59774553B81EA1691A46D9B22 ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }";
    // let g_tilde_string="G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 04BC772E1ED2D6B1BA5EE7CD66CCD9D16F5FA64A9876657FD9FB996EB60E096EF41C7BC96AA71E85C12478931EDE5C19 ] ], FP: [ BIG: [ 15D71FCB9F9BAABCCA2B622B4C2FF7658626375728546DDF32F3E4F91DDDE9DD149B40C1CA942A2AABDFC25C3443D767 ] ] ], FP2: [ FP: [ BIG: [ 0CC195A47A6A4F0329F4304C232C5C49121EFD8F53267B45BE721A7898EA863A1C4F2155A5C0ED4A31F33D126D15655D ] ], FP: [ BIG: [ 0F6BE700BDE004D83FC202E5DA8BFED0D9C04EDC76853FDEA0D1CA1B74BB11D5D69F261679ED69850BA6441D0B0F2B76 ] ] ], FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ], FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] }";
    // let X_tilde_string="G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 061CC1DB6D68F3637ABB4CAD09D66A9D3B2E4386899A877F822BE00B01554527B3A9A012619F811289AA18FBE421A7FA ] ], FP: [ BIG: [ 10C814FFD471CBF79C15C2E22421E0141331CA4DE5BD438993FEFF5002010B08350E12C41556C2ED2BF2056CE2890E17 ] ] ], FP2: [ FP: [ BIG: [ 13B2DCDEADE64CA193826F9FA72DD7879ABFC267D3AB18AA98CA5407A856CBFA001E5A066F3BA3E3CB6DF466B2B65202 ] ], FP: [ BIG: [ 10B87277E28E03FC7E3D6EDB29DB7EB898DD3D871B14BE0DF96683AFE609DC1AE72190B97F21F3055DB02941588BE798 ] ] ], FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ], FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] }";
    // let Y_tilde_string="G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 03A86B81E5F6D02C56C005324EE2BFCDB378A3D161E6E51F5C136C2C02550BC206A742D761C768AEC7BFA3BC49EA1257 ] ], FP: [ BIG: [ 16FAD4F568750B53E97C33FFA328C4804B1D0C39A3777DC455DD0AE834768969AF4D16FEF96B0D56D4628CABA7468897 ] ] ], FP2: [ FP: [ BIG: [ 04ACCCB58A0719C88541D3F8CB3278E8982CE91E040BAC146DA9E1823D4E10A9EFAF5FFFC493F609AA6B3BE6FF54F847 ] ], FP: [ BIG: [ 15ABBE3ADCE47D67CAFB345156A7B38BA3642FA04BDAEBE7EE07E8505F24688C5F4D265872AC6952C46DD71A84CEFEAD ] ] ], FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ], FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] }";

    let g=deserialize_G1_2(g_string.to_string());

    let g_tilde=deserialize_G2(g_tilde_string.to_string());

    let X_tilde=deserialize_G2(X_tilde_string.to_string());

    let Y_tilde=deserialize_G2(Y_tilde_string.to_string());

    let group_public_key: Gpk=Gpk{
        g,
        g_tilde,
        X_tilde,
        Y_tilde,
    };

    group_public_key
}


fn fixMU(signature_string : String)->((amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::group_elem_g1::G1, 
    amcl_wrapper::field_elem::FieldElement, 
    amcl_wrapper::field_elem::FieldElement)){
    let vec_one_1: Vec<&str>=signature_string.split(r#"(("#).collect();
    let vec_one_2: Vec<&str>=vec_one_1[1].split(r#", G1"#).collect();
    let G1_string1=vec_one_2[0];
    // println!("Your secret: {:?} ",G1_string1);

    let vec_two_1: Vec<&str>=vec_one_2[1].split(r#", FieldElement "#).collect();
    let G1_string2=["G1",vec_two_1[0]].join("");
    // println!("Your secret: {:?} ",G1_string2);

    let field_string1=["FieldElement ",vec_two_1[1]].join("");
    // println!("Your secret: {:?} ",field_string1);

    let vec_four_1: Vec<&str>=vec_two_1[2].split(r#", DefaultHasher"#).collect();
    let field_string2=["FieldElement ",vec_four_1[0]].join("");
    // println!("Your secret: {:?} ",field_string2);

    let signature_G1_1=deserialize_G1_2(G1_string1.to_string());

    // println!("G2_Mu: {:?}", G1_string2.to_string());
    let signature_G1_2=deserialize_G1_2(G1_string2.to_string());


    let vec: Vec<&str> = field_string1.split(r#"[ "#).collect();
    let vec2: Vec<&str> = vec[1].split(r#" ]"#).collect();
    let vec21: Vec<&str> = vec2[0].split(r#" ]"#).collect();
    let Temp_big_num=BIG::from_hex(vec21[0].to_string());
    let signature_field_element_1=FieldElement::from(Temp_big_num);

    // let signature_field_element_2
    let vec2: Vec<&str> = field_string2.split(r#"[ "#).collect();
    let vec22: Vec<&str> = vec2[1].split(r#" ]"#).collect();
    let Temp_big_num2=BIG::from_hex(vec22[0].to_string());
    let signature_field_element_2=FieldElement::from(Temp_big_num2);

    // println!("Temp_big_num: {:?}", Temp_big_num);

    // println!("vec2[0]: {:?}", vec2[0].to_string());

    // println!("Field_1: {:?}", signature_field_element_1);

    // println!("Field_2: {:?}", signature_field_element_2);
    
    let mu_1=(signature_G1_1,signature_G1_2,signature_field_element_1,signature_field_element_2);

    // println!("mu_2232323: {:?}", mu_1);

    mu_1
}


fn messageClearing(string:String)->(String){
    // println!("Your string is: {:?} ", string);
    string

}

// fn writetoFile(array: Vec<u64>) -> std::io::Result<()> {
//     let mut buffer = File::create("foo.txt")?;

//     let reference = buffer.by_ref();

//     // we can use reference just like our original buffer
//     let string="ssssdddss \n".as_bytes();
//     for x in 0..100 {
//         reference.write(string)?;
//     }
//     Ok(())
// }




use std::time::{Duration, Instant, SystemTime};



fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}


#[test]
fn test_scenario_1() {

    let mut stringzz=input_user_line();
    let mut split= stringzz.split(',');
    let vec_input = split.collect::<Vec<&str>>();
    let messagezz : String;

    if (vec_input[0]=="hashtest"){
        //check if hash is deterministic and same
        let mut hasher=DefaultHasher::new();
        let stringtest="aaabbbaaa";
        stringtest.hash(&mut hasher);
        println!("{:?}", hasher.finish());
    }
    if (vec_input[0]=="test_size"){

        // let ausize =amcl_wrapper::constants::GroupG1_SIZE;
        // println!("G1 Size: {:?} \n", ausize);
        // let ausize2 =amcl_wrapper::constants::GroupG2_SIZE;
        // println!("G2 Size {:?} \n", ausize2);
        // let ausize3 =&amcl_wrapper::constants::CurveOrder;
        // println!("G2 Size {:?} \n", ausize3);


        let count_msgs = 1;
        let label="test".as_bytes();
        let (gpk, gmsk) = GSetup(count_msgs,label);

        // User A Created
        let (upk_1, usk_1)=PKIJoin(count_msgs,label);
        let user_id=1;
        let (secret_register_1,gsk_1) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_1,usk_1);

        // // User B Created
        let (upk_2, usk_2)=PKIJoin(count_msgs,label);
        let user_id=2;
        let (secret_register_2,gsk_2) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_2,usk_2);

        let mut times = Vec::new();
        let mut times2 = Vec::new();

        // let messageA= String::from(r#"{"id":{"$oid":"601903a29d1cee5abbfb0a2c"},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#);
        ///fake gpk
        // println!("{:?}", gpk.clone());
        let string_gpk=serde_json::to_string(&gpk);
        let string_gpk_final=string_gpk.unwrap().replace("\"","");
        let vec: Vec<&str> = string_gpk_final.split(r#"g:1"#).collect();

        // println!("{:?}", vec);
        let vec_2: Vec<&str> = vec[1].split(r#" 1 "#).collect();
        let value_1=vec_2[0];
        let vec_3: Vec<&str> = vec_2[1].split(r#" 2 "#).collect();
        let value_2=vec_3[0];
        let vec_4: Vec<&str> = vec_3[1].split(r#",g_tilde:1 "#).collect();
        let value_3=vec_4[0];

        


        let extra_vec_2: Vec<&str> = string_gpk_final.split(r#",g_tilde:1 "#).collect();

        let vec_5: Vec<&str> = extra_vec_2[1].split(r#" 1 "#).collect();
        let value_4=vec_5[0];
        let value_5=vec_5[1];
        let value_6=vec_5[2];
        let vec_8: Vec<&str> = vec_5[3].split(r#" 2 "#).collect();
        let value_7=vec_8[0];
        let value_8=vec_8[1];
        let vec_10: Vec<&str> = vec_5[4].split(r#",X_tilde:1 "#).collect();
        let value_9=vec_10[0];

        let extra_vec_3: Vec<&str> = string_gpk_final.split(r#",X_tilde:1 "#).collect();


        let vec_11: Vec<&str> = extra_vec_3[1].split(r#" 1 "#).collect();
        let value_11=vec_11[0];
        let value_12=vec_11[1];
        let value_13=vec_11[2];
        let vec_15: Vec<&str> = vec_11[3].split(r#" 2 "#).collect();
        let value_14=vec_15[0];
        let value_15=vec_15[1];
        let vec_10: Vec<&str> = vec_11[4].split(r#",Y_tilde:1 "#).collect();
        let value_16=vec_10[0];

        let extra_vec_4: Vec<&str> = string_gpk_final.split(r#",Y_tilde:1 "#).collect();


        let vec_17: Vec<&str> = extra_vec_4[1].split(r#" 1 "#).collect();
        let value_17=vec_17[0];
        let value_18=vec_17[1];
        let value_19=vec_17[2];
        let vec_15: Vec<&str> = vec_17[3].split(r#" 2 "#).collect();
        let value_20=vec_15[0];
        let value_21=vec_15[1];
        let vec_23: Vec<&str> = vec_17[4].split(r#"}"#).collect();
        let value_22=vec_23[0];

        // let vec_: Vec<&str> = vec_[1].split(r#"}"#).collect();

        // println!("{:?}", value_2);

        // let assaasaassas= vec[]
        // let string_chopped=["Gpk { g: G1 { value: ECP: [ FP: [ BIG: [ ",vec_1[0]].join("");
        // let string_chopped=[string_chopped," ] ], FP: [ BIG: [ "].join("");

        // let string_chopped=["Gpk { g: G1 { value: ECP: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ] }, g_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ], FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ], FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ] ] }, X_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ], FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ], FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ] ] }, Y_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ], FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ], FP2: [ FP: [ BIG: [ "++" ] ], FP: [ BIG: [ "++" ] ] ] ] } }"].join("");



        // let de_gpk=deserialize_GroupPublicKey(string_gpk_final);
        // println!("{:?}", de_gpk);
        /////////


        ///////

        for x in 0..1 {
            //User A signs for message
            let messageA= String::from(r#"{transaction: test, tester: sen}"#);    

            let now = Instant::now();
            // let (mu_1,hash_for_tuple_1, msg_1)= GSign2(gpk.clone(),gsk_1.clone(),messageA.clone());
            let (mu_1,hash_for_tuple_1, msg_1)= GSign(gsk_1.clone(),messageA.clone());
            let new_now = Instant::now();
            let duration = new_now.duration_since(now);
            times.push(duration);

            // let message_Fake= String::from(r#"{"id":{"$oid":""},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#);    

            let now2 = Instant::now();
            // let verified_signature_1=GVerify(gpk.clone(),mu_1.clone(),hash_for_tuple_1.clone(), message_Fake.clone());
            let verified_signature_1=GVerify(gpk.clone(),mu_1.clone(),hash_for_tuple_1.clone(), messageA.clone());
            let new_now2 = Instant::now();
            let duration2 = new_now2.duration_since(now2);
            times2.push(duration2);
            println!("verified_signature_1 is: {:?} \n", verified_signature_1);
            // println!("signature_1 is: {:?} \n", mu_1);

            // println!("gsk_1:{:?}", gsk_1.clone());
            // println!("gpk:{:?}", gpk.clone());
            // println!("mu_1:{:?}", mu_1.clone());
            // println!("messageA:{:?}", messageA.clone());



            // print_type_of(&gpk.clone());
            // print_type_of(&mu_1.clone());
            // print_type_of(&hash_for_tuple_1.clone());
            // print_type_of(&messageA.clone());
        }
        println!("Gsign Time Array of Time elapsed is: {:?} \n", times);
        println!("Gverify Time Array of Time elapsed is: {:?} \n", times2);
        // println!("{:?}", mu_1);
    }

    if (vec_input[0]=="test_speed"){


        // let mut gmsk_array=Vec::new();
        //Group Created
        //number of messages used to generate pk and sk
        let count_msgs = 1;
        let label="test".as_bytes();
        let (gpk, gmsk) = GSetup(count_msgs,label);

        // User A Created
        let (upk_1, usk_1)=PKIJoin(count_msgs,label);
        let user_id=1;

        let (secret_register_1,gsk_1) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_1,usk_1);
        //Store A idenity in secret GM array
        // gmsk_array.push(secret_register_1.clone());


        // // User B Created
        // let (upk_2, usk_2)=PKIJoin(count_msgs,label);
        // let user_id=2;

        // let (secret_register_2,gsk_2) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_2,usk_2);
        // //Store B idenity in secret GM array
        // gmsk_array.push(secret_register_2.clone());



        let mut times = Vec::new();
        let mut times2 = Vec::new();

        // let messageA=r#"{"id":{"$oid":"601903a29d1cee5abbfb0a2c"},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#;
        // let messageA= String::from(r#"{"id":{"$oid":"601903a29d1cee5abbfb0a2c"},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#);    

        for x in 0..1 {
            //User A signs for message
            let messageA= String::from(r#"{"id":{"$oid":"601903a29d1cee5abbfb0a2c"},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#);    

            let now = Instant::now();
            let (mu_1,hash_for_tuple_1, msg_1)= GSign(gsk_1.clone(),messageA);
            let new_now = Instant::now();
            let duration = new_now.duration_since(now);
            times.push(duration);


            let now2 = Instant::now();

            let verified_signature_1=GVerify(gpk.clone(),mu_1.clone(),hash_for_tuple_1.clone(), msg_1.clone());
            let new_now2 = Instant::now();
            let duration2 = new_now2.duration_since(now2);
            times2.push(duration2);
            println!("verified_signature_1 is: {:?} \n", verified_signature_1);
        }
        println!("Gsign Time Array of Time elapsed is: {:?} \n", times);
        println!("Gverify Time Array of Time elapsed is: {:?} \n", times2);

        // writetoFile(times);
    }

    if (vec_input[0]=="test_sen"){
        // Vec<(usize,amcl_wrapper::group_elem_g1::G1,Signature,amcl_wrapper::group_elem_g2::G2,DefaultHasher)>
        let mut gmsk_array=Vec::new();
        //Group Created
        //number of messages used to generate pk and sk
        let count_msgs = 1;
        let label="test".as_bytes();
        let (gpk, gmsk) = GSetup(count_msgs,label);

        // User A Created
        let (upk_1, usk_1)=PKIJoin(count_msgs,label);
        let user_id=1;



        // check DS Encryption
        // let msg = FieldElementVector::random(count_msgs);


        // let start5 = Instant::now();
        // let sign_usk_1=Signature::new(msg.as_slice(), &usk_1, &upk_1).unwrap();
        // let duration5 = start5.elapsed();
        // println!("Time elapsed in DSsigning of A is: {:?}", duration5);


        // let start6 = Instant::now();
        // let check=sign_usk_1.verify(msg.as_slice(),&upk_1.clone()).unwrap();
        // let duration6 = start6.elapsed();
        // println!("Time elapsed in DSverifying of A is: {:?}", duration6);
        // check DS Encryption




        // let mut stringzz= input_user();
        // println!("You typed: {}",stringzz);


        let (secret_register_1,gsk_1) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_1,usk_1);
        //Store A idenity in secret GM array
        gmsk_array.push(secret_register_1.clone());


        // User B Created
        let (upk_2, usk_2)=PKIJoin(count_msgs,label);
        let user_id=2;


        // check DS Encryption
        // let start7 = Instant::now();
        // let sign_usk_2=Signature::new(msg.as_slice(), &usk_2, &upk_2).unwrap();
        // let duration7 = start7.elapsed();
        // println!("Time elapsed in DSsigning of A is: {:?}", duration7);


        // let start8 = Instant::now();
        // let check=sign_usk_2.verify(msg.as_slice(), &upk_2.clone()).unwrap();
        // let duration8 = start8.elapsed();
        // println!("Time elapsed in DSverifying of A is: {:?}", duration8);
        // check DS Encryption




        let (secret_register_2,gsk_2) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_2,usk_2);
        //Store B idenity in secret GM array
        gmsk_array.push(secret_register_2.clone());


        // // Test increase in group size
        // let (upk_3, usk_3)=PKIJoin(count_msgs,label);
        // let user_id=3;
        // let (secret_register_3,gsk_3) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_3,usk_3);
        // gmsk_array.push(secret_register_3.clone());

        // let (upk_4, usk_4)=PKIJoin(count_msgs,label);
        // let user_id=4;
        // let (secret_register_4,gsk_4) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_4,usk_4);
        // gmsk_array.push(secret_register_4.clone());

        // let (upk_5, usk_5)=PKIJoin(count_msgs,label);
        // let user_id=5;
        // let (secret_register_5,gsk_5) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_5,usk_5);
        // gmsk_array.push(secret_register_5.clone());

        let (upk_6, usk_6)=PKIJoin(count_msgs,label);
        let user_id=6;
        let (secret_register_6,gsk_6) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_6,usk_6);
        gmsk_array.push(secret_register_6.clone());


        // let (upk_7, usk_7)=PKIJoin(count_msgs,label);
        // let user_id=7;
        // let (secret_register_7,gsk_7) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_7,usk_7);
        // gmsk_array.push(secret_register_7.clone());


        // let (upk_8, usk_8)=PKIJoin(count_msgs,label);
        // let user_id=8;
        // let (secret_register_8,gsk_8) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_8,usk_8);
        // gmsk_array.push(secret_register_8.clone());


        // let (upk_9, usk_9)=PKIJoin(count_msgs,label);
        // let user_id=9;
        // let (secret_register_9,gsk_9) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_9,usk_9);
        // gmsk_array.push(secret_register_9.clone());

        // let (upk_10, usk_10)=PKIJoin(count_msgs,label);
        // let user_id=10;
        // let (secret_register_10,gsk_10) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_10,usk_10);
        // gmsk_array.push(secret_register_10.clone());

        // let (upk_11, usk_11)=PKIJoin(count_msgs,label);
        // let user_id=11;
        // let (secret_register_11,gsk_11) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_11,usk_11);
        // gmsk_array.push(secret_register_11.clone());


        // let (upk_12, usk_12)=PKIJoin(count_msgs,label);
        // let user_id=12;
        // let (secret_register_12,gsk_12) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_12,usk_12);
        // gmsk_array.push(secret_register_12.clone());

        // let (upk_13, usk_13)=PKIJoin(count_msgs,label);
        // let user_id=13;
        // let (secret_register_13,gsk_13) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_13,usk_13);
        // gmsk_array.push(secret_register_13.clone());

        // let (upk_14, usk_14)=PKIJoin(count_msgs,label);
        // let user_id=14;
        // let (secret_register_14,gsk_14) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_14,usk_14);
        // gmsk_array.push(secret_register_14.clone());   
        
        // let (upk_15, usk_15)=PKIJoin(count_msgs,label);
        // let user_id=15;
        // let (secret_register_15,gsk_15) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_15,usk_15);
        // gmsk_array.push(secret_register_15.clone());

        // let (upk_16, usk_16)=PKIJoin(count_msgs,label);
        // let user_id=16;
        // let (secret_register_16,gsk_16) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_16,usk_16);
        // gmsk_array.push(secret_register_16.clone());



        // let (upk_x, usk_x)=PKIJoin(count_msgs,label);
        // let user_id=x;
        // let (secret_register_x,gsk_x) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_x,usk_x);
        // gmsk_array.push(secret_register_x.clone());


        let mut times = Vec::new();

        for x in 0..100 {
            // let messageA=r#"{"id":{"$oid":"601903a29d1cee5abbfb0a2c"},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#;
            let messageA= String::from(r#"{"id":{"$oid":"601903a29d1cee5abbfb0a2c"},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#);
            //User A signs for message
            let start = Instant::now();
            let (mu_1,hash_for_tuple_1, msg_1)= GSign(gsk_1.clone(),messageA);
            let duration = start.elapsed();
            // println!("Time elapsed in Gsigning of A is: {:?}", duration);
            times.push(duration);
        }
        println!("Time elapsed in Gsigning of A is: {:?}", times);

        // let messageA=r#"{"id":{"$oid":"601903a29d1cee5abbfb0a2c"},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#;
        // //User A signs for message
        // let start = Instant::now();
        // let (mu_1,hash_for_tuple_1, msg_1)= GSign(gsk_1.clone(),messageA);
        // let duration = start.elapsed();
        // println!("Time elapsed in Gsigning of A is: {:?}", duration);

        // let start2 = Instant::now();
        // let verified_signature_1=GVerify(gpk.clone(),mu_1.clone(),hash_for_tuple_1.clone(), msg_1.clone());
        // let duration2 = start2.elapsed();
        // println!("Time elapsed in Gverifying of A is: {:?}", duration2);

        // //User B signs for message
        // let start3 = Instant::now();
        // let (mu_2,hash_for_tuple_2, msg_2)= GSign(gsk_2.clone(), messageA);
        // let duration3 = start3.elapsed();
        // println!("Time elapsed in Gsigning of B is: {:?}", duration3);

        // let start4 = Instant::now();
        // let verified_signature_2=GVerify(gpk.clone(),mu_2.clone(),hash_for_tuple_2.clone(), msg_2.clone());
        // let duration4 = start4.elapsed();
        // println!("Time elapsed in Gverifying of B is: {:?}", duration4);


        // // //User B signs for message
        // let start5 = Instant::now();
        // let (mu_6,hash_for_tuple_6, msg_6)= GSign(gsk_6.clone(),messageA);
        // let duration5 = start5.elapsed();
        // println!("Time elapsed in Gsigning of Z is: {:?}", duration5);

        // let start6 = Instant::now();
        // let verified_signature_6=GVerify(gpk.clone(),mu_6.clone(),hash_for_tuple_6.clone(), msg_6.clone());
        // let duration6 = start6.elapsed();
        // println!("Time elapsed in Gverifying of Z is: {:?}", duration6);





        // // who signed mu_1,hash_for_tuple_1,msg_1?
        // println!("Who signed this? {:?}", msg_1.clone());
        // GOpen(gpk.clone(),gmsk_array.clone(),mu_1.clone(),hash_for_tuple_1.clone(),msg_1.clone());
        // // who signed mu_2,hash_for_tuple_2,msg_2?
        // println!("Who signed this? {:?}", msg_2.clone());
        // GOpen(gpk.clone(),gmsk_array.clone(),mu_2.clone(),hash_for_tuple_2.clone(),msg_2.clone());


    }

    // if (vec_input[0]=="GSign"){


    //     let string_ski = vec_input[1].replace("comma", ",");

    //     let string_mess = vec_input[2].replace("comma", ",");
    //     let vec_message: Vec<&str>=string_mess.split(r#"{"#).collect();
    //     let messageB = [r#"{"#,vec_message[1]].join("");



    //     //Extract and Format Field Element String
    //     let vec_one_1: Vec<&str>=string_ski.split(r#" ] }, (G1 { "#).collect();
    //     let stitch_string1=[vec_one_1[0], r#"] }"#].join("").replace("(FieldElement","FieldElement").replace(" ","").replace(":",": ");

    //     //Extract and G1 Element String
    //     let vec_one_21: Vec<&str>=string_ski.split(r#" ] }, ("#).collect();
    //     let vec_one_211: Vec<&str>=vec_one_21[1].split(r#", G1 { value: ECP:"#).collect();
    //     let stitch_string21=vec_one_211[0].replace(" ","");

    //     let vec_two_22: Vec<&str> =vec_one_21[1].split(r#" ] ] ] },"#).collect();
    //     let vec_two_221: Vec<&str> =vec_two_22[1].split(r#"), '"#).collect();
    //     let stitch_string22=vec_two_221[0].replace(" ","");

    //     //Extract and GT Element String
    //     let stitch_string3=vec_two_221[1].replace(r#"') "#,"");
        
    //     // //////group secret key
    //     //deserialize FieldElement /////////
    //     // let messageA= String::from(r#"{"id":{"$oid":"601903a29d1cee5abbfb0a2c"},"asset":{"data":{}},"id":"928cd26da5dcc88efc2b3c5d9a5ed0d8416bca332fae2c152c4d9444fff364d1","inputs":[{"fulfillment":"pGSAIL2a-U9jW0N1GWPhx9pOU2TPp-IDSGM3QwG_qh-XZID8gUC34dkT3XXkRqAybzP7CE1Z4LnGfFsQUKtR8OtbjXD8HxPSuYmMpwH3mXbrPGGHyC-5RMjDNcuWuON1wYzKgVEC","fulfills":null,"owners_before":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"operation":"PRE_REQUEST","outputs":[{"amount":"1","condition":{"details":{"public_key":"Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs","type":"ed25519-sha-256"},"uri":"ni:///sha-256;AXP8Er6zU1lEl4KVPfnoaLnh3NTojjCVR8UdAxpX6xo?fpt=ed25519-sha-256&cost=131072"},"public_keys":["Dm97eMH8VQpaF19fxqahxn92sko4gVRB8NCEjCkztUgs"]}],"version":"2.0"}"#);
    //     // let string_1 ="FieldElement{value: BIG: [0000000000000000000000000000000030313950CD9853C985B0F474BFB2C82DFA2900EE0AE48DA87768D1D88A07B5BE]}";
    //     let vec: Vec<&str> = stitch_string1.split(r#"["#).collect();

    //     let vec2: Vec<&str> = vec[1].split(r#"]"#).collect();


    //     let Temp_big_num=BIG::from_hex(vec2[0].to_string());

    //     let field_element = FieldElement::from(Temp_big_num);
    //     // println!("field_element: {:?}", field_element);
    //     //deserialize G1//////
    //     // let string_21="G1{value:ECP:[FP:[BIG:[0F14C406809B10B4C15144BB87CF0BD1017273A17FCC10107F3C8DFDA475195AD8150539A58030706C56BFBE308DC589]],FP:[BIG:[00224B1552B3263C88E87B320DD5F8A42BEE25680CDED3C068495C70E64CC854AA214BECD05CFBDAAA6FB94ABC346D31]],FP:[BIG:[13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84]]]}";
    //     let G1_21_element=deserialize_G1(stitch_string21.to_string());


    //     // let string_22="G1{value:ECP:[FP:[BIG:[321A221AD4CBA9A3A6941B72C4DA686DECDD7170B9F06E42F5871661B5A18E2FEE815A1D5FA128D0D70DDE3AAB021754]],FP:[BIG:[2C7B443CDA1EEC37C17AEA4D623E40E1AE17B240345D36CC57FD0EBAEF874DE9BFBB141D8D7A128EF754F04AFA8B1C22]],FP:[BIG:[2F4311D68E7ED02064B0D9BEE9E28D869D4104135220A9E28FB4902DAE3D87B0E97DA23DC7B63B3F76CF030C7F21B6D9]]]}";        
    //     let G1_22_element=deserialize_G1(stitch_string22.to_string());




    //     //deserialize GT
    //     // let string_3="1 13D22ABF7C9AAA01BA0B2D748F0856DBD91D760548C0A048BEBF3E846E6566BE9507C42383E795498B3EDDA4766E2C0E 1 102A2FB0B587560778041E6CB360BE99A68F87AE632E43A7EB7565D07E822DF16CBED657A2F81F832D09F9CAD8158428 1 079FEB234A677326B0A7212F5FB5FCE6A53E069896CBAF3BD428C0F421F368B88C83DEBCEB2AEA2E76C596D3CF80D413 1 15F4B0166F15270E45667E4E622C17CFB8B8A1757442EA60D292CF669142318A688773DA1389A298AC3DE3C9FB2020A6 1 01AB217183B7187B644024251C7F912EC11290B6B104EA8F2367DFD59E74734E30C72302D042DB28A14A83490D05AD85 1 14DB2039D20250DF1600DE3CA836D9856000754EE0CEFF1ECD9DD40C38A1FB6DE5C5A1106FBBC174897F92C116EAF1DB 1 048986A0245A63226E3901ABCB59FCDBD5C02409B0BDAE46A20FD55A799FDAE16AC5E16A391F341CC179DCA96D8EC299 1 16F8A5E4A6EAB9471BAA6E10587D262B3BB622FA094E676681E6F772D020795EE5F9F286C332B79DF0ABD250507A3A71 1 0530000AF31E902AF967CF16BC0E5D241B753A8E295BA3DE66A080461C86672FEF02289F09A5538895DE01244C2FEFEA 1 16906A08D248C92207DB729C290584C561D680CF4662DF9131E0F4213DF7F753064015E6DC82C4B9290D824CB2595BCC 1 0258FBA29A293BCCD94AFE52063D796B3144FDF668E66A5EF689EA20D52A28FF49760D67ABE47FEE38CD2DA8693B68BA 1 0EC9A613C06F19DC107F62BE4C4B9F3AF5A93EAF4EA73ADF60504CB535F44E4277335AAF8FD28C8B45E180E61E7907A1";

    //     // println!("3 is: {:?} ", string_3);
    //     // println!("3 is: {:?} ", stitch_string3);

    //     // println!("Your ski is: {:?} ", (field_element,(G1_21_element,G1_22_element),string_3));

    //     let gsk_1=(field_element,(G1_21_element,G1_22_element),stitch_string3);

    //     messagezz = messageClearing(string_mess);
    //     // println!("Your message is: {:?} ",messagezz);





    //     // //cube of test
    //     // let count_msgs11 = 1;
    //     // let label11="test".as_bytes();
    //     // let (gpk11, gmsk11) = GSetup(count_msgs11,label11);
    //     // let (upk_111, usk_111)=PKIJoin(count_msgs11,label11);
    //     // let user_id11=1;
    //     // let (secret_register_111,gsk_11) = GJoin (user_id11,gpk11.clone(),gmsk11.clone(), upk_111,usk_111);
    //     // let (mu_11,hash_for_tuple_11, msg_11)= GSign(gsk_11.clone(),messagezz.clone());
    //     // let verified_signature_11=GVerify(gpk11.clone(),mu_11.clone(),hash_for_tuple_11.clone(), messagezz.clone());
    //     // println!("verified_signature_1 is: {:?} \n", verified_signature_11);
    //     // /////////////



    //     // ////////////////////////////////////
    //     // ////////////turn back on to reveal signature
    //     // let (mu_1,hash_for_tuple_1, msg_1)= GSign(gsk_1.clone(),messagezz.clone());
    //     // println!("Signature is: {:?} \n", (mu_1,hash_for_tuple_1, msg_1));
    //     // ////////////turn back on to reveal signature
    //     // ////////////////////////////////////


    //     // ///////////////////////////////testing verify
    //     // let messageA= String::from(r#"{transaction: test, tester: sen}"#);
    //     // if (messagezz.clone()==messageA){
    //     //     println!("equal");
    //     // }
    //     // else {
    //     //     println!("not equal");
    //     //     println!("{:?}", messageA);
    //     //     println!("{:?}", messagezz);
    //     // }

    //     let string_gpk = vec_input[3].replace("comma", ",");
    //     let group_public_key=deserialize_GroupPublicKey(string_gpk);

    //     let (mu_1,hash_for_tuple_1, msg_1)= GSign2(group_public_key.clone(),gsk_1.clone(),messagezz.clone());
    //     // println!("Signature is: {:?} \n", (mu_1,hash_for_tuple_1, msg_1));

    //     // println!("gsk_1 is: {:?} \n", gsk_1.clone());
    //     // println!("Gpk is: {:?} \n", group_public_key);
    //     // println!("Signature is: {:?} \n", mu_1);
    //     // println!("Hasher is: {:?} \n", hash_for_tuple_1);


    //     // print_type_of(&group_public_key.clone());
    //     // print_type_of(&mu_1.clone());
    //     // print_type_of(&hash_for_tuple_1.clone());
    //     // print_type_of(&messagezz.clone());


    //     let teststringer = "Gpk { g: G1 { value: ECP: [ FP: [ BIG: [ 176827B765AC12FFF95F1191D8C8AA2EC805EAF1BCE7710E2F6ED3AACAF125E1FAC1E6AC9C32B8925D77E26A5C9CAEEC ] ]comma FP: [ BIG: [ 0814B11A08E2E3014E958D84B74D0DAB030541EC7794EAC69D052CDA6926C9A2C53B63A59774553B81EA1691A46D9B22 ] ]comma FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }comma g_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 04BC772E1ED2D6B1BA5EE7CD66CCD9D16F5FA64A9876657FD9FB996EB60E096EF41C7BC96AA71E85C12478931EDE5C19 ] ]comma FP: [ BIG: [ 15D71FCB9F9BAABCCA2B622B4C2FF7658626375728546DDF32F3E4F91DDDE9DD149B40C1CA942A2AABDFC25C3443D767 ] ] ]comma FP2: [ FP: [ BIG: [ 0CC195A47A6A4F0329F4304C232C5C49121EFD8F53267B45BE721A7898EA863A1C4F2155A5C0ED4A31F33D126D15655D ] ]comma FP: [ BIG: [ 0F6BE700BDE004D83FC202E5DA8BFED0D9C04EDC76853FDEA0D1CA1B74BB11D5D69F261679ED69850BA6441D0B0F2B76 ] ] ]comma FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ]comma FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] }comma X_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 0EF0C3096DA5910C3B4671DE17A34CD9A1417A3509AED87E78BA7CCA592F71BD7B218F9B08D2BA102A79316BA0436E71 ] ]comma FP: [ BIG: [ 16852C5DAF5D9C0BDBD6CE2C8D9F23D0CF372AE59CD8AB643D260FBEFFB907DC31C52A9C43F61D9ABEF43075E7BDFA85 ] ] ]comma FP2: [ FP: [ BIG: [ 11C97BEE27F133FD2140894F726AA7B2C55D37E592B1F84E21A4097406CC084B75DB04F90620AD9A326943D913700CC8 ] ]comma FP: [ BIG: [ 0FA9357A8267E8BBC4A57C8ED0ABA6E7E7F6B5C293BE08530C6F35D453700A2221C49E3818336319776AB96DB185E3A1 ] ] ]comma FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ]comma FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] }comma Y_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 15FB546F85D556DC43212BC33FBAD8BDB8A7FE0D996D141ED8B638FFA2273AB501F3C7B56BB5A382FAD6090C27893493 ] ]comma FP: [ BIG: [ 1289DCCA838A49C0A6CEB68F3625A0B80D5DB576C3E2947C07B6B7CA020BB06F08E1F13A57BB10CC4547339FDA4260D8 ] ] ]comma FP2: [ FP: [ BIG: [ 0EA39A3C1CC29B5AA33DC912DFC8AAE348AA68575936BB73FB4E736951BD361BD84DE6D3FF68FBE153C81E77D4CCFC15 ] ]comma FP: [ BIG: [ 0BB110EE5CFA3493893EFA58ED9694B263131D7079AAE67760A3274BC4A82941D546041D3EF22A8FB306B7C1D65E23DF ] ] ]comma FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ]comma FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] } } ";
    //     let testmumu = "((G1 { value: ECP: [ FP: [ BIG: [ 08B04FBD715E3553E7DCF2D79979EC0F944CF26E58B71ED9A050B5FB11620F821B8CEFE43C97D04CA1AA4D6A1A88A3A4 ] ], FP: [ BIG: [ 19EB8E3680C9804FA9DE93F2F9405359A799C604A68DE6AFFAE548CC3D76355AE2F01A0AABB900F2130EDC2EC62DF11F ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }, G1 { value: ECP: [ FP: [ BIG: [ 145E6AB325A5A1A7BC645E2CE6966451C9D4B3231D02C39E92A250657FEB7B7F57BF36761710D0BB9766E72FB992CAC7 ] ], FP: [ BIG: [ 021FF7CB8F6BC6ADA04FCF32150972803694A9778ECD7A8254DDDD9B1B592B1351586A411016DCEB02520411FDB718A7 ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }, FieldElement { value: BIG: [ 000000000000000000000000000000001A9F40C9741E50D5F9FC67E5751E6F53174F249F7BBD9F842BE88307233B5233 ] }, FieldElement { value: BIG: [ 00000000000000000000000000000000251B78E84AD0E95F6738E5544315729B9AE0CDD9E54CFF4B0562A1FE1ABD4561 ] }) ";
    //     // let testmumu2 = "((G1 { value: ECP: [ FP: [ BIG: [ 08B04FBD715E3553E7DCF2D79979EC0F944CF26E58B71ED9A050B5FB11620F821B8CEFE43C97D04CA1AA4D6A1A88A3A4 ] ], FP: [ BIG: [ 19EB8E3680C9804FA9DE93F2F9405359A799C604A68DE6AFFAE548CC3D76355AE2F01A0AABB900F2130EDC2EC62DF11F ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }, G1 { value: ECP: [ FP: [ BIG: [ 145E6AB325A5A1A7BC645E2CE6966451C9D4B3231D02C39E92A250657FEB7B7F57BF36761710D0BB9766E72FB992CAC7 ] ], FP: [ BIG: [ 021FF7CB8F6BC6ADA04FCF32150972803694A9778ECD7A8254DDDD9B1B592B1351586A411016DCEB02520411FDB718A7 ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }, FieldElement { value: BIG: [ 000000000000000000000000000000001A9F40C9741E50D5F9FC67E5751E6F53174F249F7BBD9F842BE88307233B5233 ] }, FieldElement { value: BIG: [ 00000000000000000000000000000000251B78E84AD0E95F6738E5544315729B9AE0CDD9E54CFF4B0562A1FE1ABD4561 ] }) ";

    //     let testmeage = "{transaction: testcomma tester: sen}";

    //     let string_gpk = teststringer.replace("comma", ",");
    //     let mu_12 = fixMU(testmumu.to_string());
    //     // let mu_13 = fixMU(testmumu2.to_string());


    //     let s = FieldElement::random(); 
    //     // let s = FieldElement::one();

    //     let hex_repr = mu_1.0.to_hex();
    //     // println!("hex_repr:{:?}\n", hex_repr);



    //     println!("G1_mu_1 scalar mult test: {:?} \n", mu_1.0.scalar_mul_variable_time(&s));
    //     println!("G1_mu_2 scalar mult test: {:?} \n", mu_12.0.scalar_mul_variable_time(&s));

    //     let mu_3 = G1::from_hex(hex_repr).unwrap(); 
    //     println!("G1_mu_3 scalar mult test: {:?} \n", mu_3.scalar_mul_variable_time(&s));


    //     let mu_4 = G1::from_hex("1 08B04FBD715E3553E7DCF2D79979EC0F944CF26E58B71ED9A050B5FB11620F821B8CEFE43C97D04CA1AA4D6A1A88A3A4 1 19EB8E3680C9804FA9DE93F2F9405359A799C604A68DE6AFFAE548CC3D76355AE2F01A0AABB900F2130EDC2EC62DF11F 2 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84".to_string()).unwrap(); 
    //     println!("G1_mu_4 scalar mult test: {:?} \n", mu_4.scalar_mul_variable_time(&s));

    //     // g.scalar_mul_variable_time(&a);

    //     if (mu_1==mu_12){
    //         println!("equal!");
    //     }
    //     // println!("G1_mu_4 scalar mult test: {:?} \n", mu_13.0.scalar_mul_variable_time(&s));


    //     // let temp14: String = mu_1.as_str();
    //     // let mu_14 = fixMU(temp14.to_string());
    //     // println!("G1_mu_5 scalar mult test: {:?} \n", mu_14.0.scalar_mul_variable_time(&s));


    //     // println!("mu_1 is: {:?} \n", mu_1);
    //     // println!("mu_12 is: {:?} \n", mu_12);
    //     // print_type_of(&mu_1);
    //     // print_type_of(&mu_12);

    //     let group_public_key2=deserialize_GroupPublicKey(string_gpk);
    //     let messagezz12 = messageClearing(testmeage.replace("comma", ",").to_string());
    //     let verified_signature_1=GVerify2(group_public_key2.clone(),mu_1.clone(),hash_for_tuple_1.clone(), messagezz12.clone());
    //     println!("verified_signature_1 is: {:?} \n", verified_signature_1);


    //     let verified_signature_2=GVerify2(group_public_key2.clone(),mu_12.clone(),hash_for_tuple_1.clone(), messagezz12.clone());
    //     println!("verified_signature_2 is: {:?} \n", verified_signature_2);

    //     // let verified_signature_3=GVerify2(group_public_key2.clone(),mu_13.clone(),hash_for_tuple_1.clone(), messagezz12.clone());
    //     // println!("verified_signature_3 is: {:?} \n", verified_signature_3);


    //     // let verified_signature_4=GVerify2(group_public_key2.clone(),mu_1.clone(),hash_for_tuple_1.clone(), messagezz12.clone());
    //     // println!("verified_signature_4 is: {:?} \n", verified_signature_4);





    //     // let verified_signature_1=GVerify2(group_public_key.clone(),mu_1.clone(),hash_for_tuple_1.clone(), messagezz.clone());
    //     // println!("verified_signature_1 is: {:?} \n", verified_signature_1);
    //     // ///////////////////////////////testing verify










    //     /////////////////// serialization reference can be deleted -sen


    //     // let converted_string_3=string_3.replace(r#"[[["#, "1 ").replace(r#"]],[["#, " 1 ").replace(r#"],["#, " 1 ").replace(r#","#, " 1 ").replace(r#"]]]"#,"");;

    //     // let stringaaaaaa="1 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000A 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 1 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

    //     // let test =GT::one();

    //     // let test2=test.to_hex();

    //     // let gt_convert=GT::from_hex(test2.to_string());

    //     // println!("Your test is: {:?} ", test);
    //     // println!("Your test2 is: {:?} ", test2);
    //     // println!("Your gt_convert is: {:?} ", gt_convert.unwrap());
    //     /////

    //     // let toHEXConvert=0x008046404E728D91F5E13581802561FECB6EC32DF1437423EEDFC1BF65E53CBB027C06A70FADBC720D6722AE7FB0D4C0.to_hex();
        


    //     // // let gt_rad=GT::one();

    //     // // let gt_rad_string=gt_rad.to_hex();

    //     // // let rad_back=GT::from_hex(gt_rad_string.to_string());

    //     // // println!("Your gt_rad is: {:?} ", gt_rad);
    //     // // println!("Your gt_rad_string is: {:?} ", gt_rad_string);
    //     // // println!("Your rad_back is: {:?} ", rad_back.unwrap());

    //     // // let gt_rad=GT::one();

    //     // // let gt_rad_string=gt_rad.to_bytes();


    //     // // [[[008046404E728D91F5E13581802561FECB6EC32DF1437423EEDFC1BF65E53CBB027C06A70FADBC720D6722AE7FB0D4C0,01509FDD094F3EEA55FA9675DEA72DD3EF06C8AAE274F53EB2E99F3D36578A6538A5007655AB4FC52C0CF623345AA133],[0181D9522DAA6B05078EA94852A586FE531D04DE3948072E9095D300FC9BB2B3D619FB89BBFBAA7926EEB0C9D127FAA6,0FEBB21B9E14F9A6FFE3EE874025D470607E0ED024A511197D0E653F3EDA97B4EBE942EC8F82A98CDF76C0418FE05B44]],[[10CEB954A243B245EBFD05CECEBFB8FABE61C711826412D68D7DB0D1EDA4CC8EBB6F0DE40D7F6A381E4A88C650F288DF,184715E1684D10152B53A2EC41ED7FB5CA731B6191EDEE8D0099534C85DF1C64B26A03967C708BD42B1AD67CC0AE1F8F],[19EBBD2A2FD9115CD2DC5CB1BA8F030537C26661F8F995182CAC0A4C05145A228FACCE62DE77B7A786A4107CAF72BF8C,083BB801C161CC12BED81E852095532BA691DE2D14E54BA911EA867E721825A8827AA911DA9A0956E5887693D1868383]],[[08EA96BF7FEC50E2DA40739B7062D3EF3C4FA00CC7ABCA2BCF840270FFB582CE549E62ADA1E606D4D8271B9CFAF67532,1780FB5C14F6A8B875FA72101B1955B5E142A11AD2E5B923A06D03584D4384A2CC4B15B8A6AD10252FC65C443F76CD15],[014DB4C8CB942210152739B4C4416C9109EB9E3FB92527569F4166D478C85A3DCB72F5A2CB0BE9A7959A05B4FF55DB32,19FFF569FF72D009903861BEEC4F911528FDDA7F6467E6335305A93F9F0756925F4C6BA673076C5873449611FE756B5D]]]

    //     // let mut gt_rad_string_fake = vec![0, 8, 0, 4, 6, 4, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    //     // let rad_back=GT::from_bytes(&gt_rad_string_fake);

    //     // // println!("Your gt_rad is: {:?} ", gt_rad);

    //     // // println!("Your length is: {:?} ", gt_rad_string_fake.len());
    //     // // print_type_of(&gt_rad_string_fake);
    //     // // print_type_of(&gt_rad_string_fake[0]);

    //     // // print_type_of(&gt_rad_string);
    //     // // print_type_of(&gt_rad_string[0]);
    //     // // println!("Your length is: {:?} ", gt_rad_string.len());


    //     // // println!("Your gt_rad_bytes is: {:?} ", gt_rad);

    //     // println!("Your rad_back is: {:?} ", rad_back);



    //     // // println!("Your ski bytes is: {:?} ", (field_element,(G1_21_element,G1_22_element),gt_convert.unwrap()));


    //     // let a = FieldElement::one();

    //     // println!("Your Fieldelement is: {:?} ", a);

    //     // let b =a.to_bytes();


    //     // println!("Your Fieldelement bytes is: {:?} ", b);

    //     // let c=FieldElement::from_bytes(&b);

    //     // println!("Your Fieldelement bytes is: {:?} ", c);



    //     // let leg=[i64::from(0)];

    //     //too large
    //     // let xx=BIG::new_int(21797987816160111514321859456480535143630098808925005577649878088290990011838);
    //     // let xx=BIG { w: [i64::from(21797987816160111514321859456480535143630098808925005577649878088290990011838); 7],};
    //     // let y=BIG::new();

    //     // let x= BIG::new_copy(&y);


    //     // let tester=GT::from_hex(string_3.to_string());
    //     // let tester=GT::new();

    //     // let testfp=FP12::new();

    //     // let tessss=tester.to_hex();

    //     /////////////////////////


    //     // let gsk_string=vec_input[1].split(',');

    //     // // amcl_wrapper::field_elem::FieldElement
    //     // let string_1="FieldElement{value: BIG: [0000000000000000000000000000000030313950CD9853C985B0F474BFB2C82DFA2900EE0AE48DA87768D1D88A07B5BE]}";
    //     // let firstpart: amcl_wrapper::field_elem::FieldElement = serde_json::from_str(string_1).unwrap();
        
    //     // // (amcl_wrapper::group_elem_g1::G1, amcl_wrapper::group_elem_g1::G1)
    //     // let string_21="G1{value:ECP:[FP:[BIG:[0F14C406809B10B4C15144BB87CF0BD1017273A17FCC10107F3C8DFDA475195AD8150539A58030706C56BFBE308DC589]],FP:[BIG:[00224B1552B3263C88E87B320DD5F8A42BEE25680CDED3C068495C70E64CC854AA214BECD05CFBDAAA6FB94ABC346D31]],FP:[BIG:[13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84]]]}";
    //     // let string_22="G1{value:ECP:[FP:[BIG:[321A221AD4CBA9A3A6941B72C4DA686DECDD7170B9F06E42F5871661B5A18E2FEE815A1D5FA128D0D70DDE3AAB021754]],FP:[BIG:[2C7B443CDA1EEC37C17AEA4D623E40E1AE17B240345D36CC57FD0EBAEF874DE9BFBB141D8D7A128EF754F04AFA8B1C22]],FP:[BIG:[2F4311D68E7ED02064B0D9BEE9E28D869D4104135220A9E28FB4902DAE3D87B0E97DA23DC7B63B3F76CF030C7F21B6D9]]]}";
    //     // let secondpart_1 : amcl_wrapper::group_elem_g1::G1 = serde_json::from_str(string_21).unwrap();
    //     // let secondpart_2 : amcl_wrapper::group_elem_g1::G1 = serde_json::from_str(string_22).unwrap();
    //     // let secondpart=(secondpart_1,secondpart_2);

    //     // // amcl_wrapper::extension_field_gt::GT
    //     // let string_3="[[[008046404E728D91F5E13581802561FECB6EC32DF1437423EEDFC1BF65E53CBB027C06A70FADBC720D6722AE7FB0D4C0,01509FDD094F3EEA55FA9675DEA72DD3EF06C8AAE274F53EB2E99F3D36578A6538A5007655AB4FC52C0CF623345AA133],[0181D9522DAA6B05078EA94852A586FE531D04DE3948072E9095D300FC9BB2B3D619FB89BBFBAA7926EEB0C9D127FAA6,0FEBB21B9E14F9A6FFE3EE874025D470607E0ED024A511197D0E653F3EDA97B4EBE942EC8F82A98CDF76C0418FE05B44]],[[10CEB954A243B245EBFD05CECEBFB8FABE61C711826412D68D7DB0D1EDA4CC8EBB6F0DE40D7F6A381E4A88C650F288DF,184715E1684D10152B53A2EC41ED7FB5CA731B6191EDEE8D0099534C85DF1C64B26A03967C708BD42B1AD67CC0AE1F8F],[19EBBD2A2FD9115CD2DC5CB1BA8F030537C26661F8F995182CAC0A4C05145A228FACCE62DE77B7A786A4107CAF72BF8C,083BB801C161CC12BED81E852095532BA691DE2D14E54BA911EA867E721825A8827AA911DA9A0956E5887693D1868383]],[[08EA96BF7FEC50E2DA40739B7062D3EF3C4FA00CC7ABCA2BCF840270FFB582CE549E62ADA1E606D4D8271B9CFAF67532,1780FB5C14F6A8B875FA72101B1955B5E142A11AD2E5B923A06D03584D4384A2CC4B15B8A6AD10252FC65C443F76CD15],[014DB4C8CB942210152739B4C4416C9109EB9E3FB92527569F4166D478C85A3DCB72F5A2CB0BE9A7959A05B4FF55DB32,19FFF569FF72D009903861BEEC4F911528FDDA7F6467E6335305A93F9F0756925F4C6BA673076C5873449611FE756B5D]]]";
    //     // let thirdpart: amcl_wrapper::extension_field_gt::GT = serde_json::from_str(string_3).unwrap();

    //     // let gsk_1 = (firstpart.clone(),secondpart.clone(),thirdpart.clone());
    //     // // let gsk_1= vec_input[1].parse::<usize>().unwrap();
    //     // let gpk=vec_input[2].as_bytes();
    //     // let message=vec_input[3].as_bytes();
    //     // let (mu_1,hash_for_tuple_1, msg_1)= GSign(gsk_1.clone(),"I require 10 boeing 747");
    //     // // let verified_signature_1=GVerify(gpk.clone(),mu_1.clone(),hash_for_tuple_1.clone(), msg_1.clone());
    //     // println!("Your signature is: {:?} ",(mu_1,hash_for_tuple_1, msg_1));
    // }

    // println!("You typed: {}",vec_input[1]);

    if (vec_input[0]=="GVerify"){


        let string_gpk = vec_input[1].replace("comma", ",");
        let group_public_key=deserialize_GroupPublicKey(string_gpk);
        // println!("{:?}", group_public_key);



        // let signature_string = vec_input[2].replace("comma", ",");
        // println!("{:?}", signature_string);

        let signature_string=vec_input[2].replace("\'","");
        let vec_one_1: Vec<&str>=signature_string.split(r#"comma "#).collect();

        // println!("{:?}", vec_one_1);

        let mu_1_1_hex_repr=vec_one_1[0].replace("((","");
        let mu_1_1=G1::from_hex(mu_1_1_hex_repr).unwrap(); 

        // println!("{:?}", vec_one_1[1]);
        let mu_1_2_hex_repr=vec_one_1[1].to_string();
        let mu_1_2=G1::from_hex(mu_1_2_hex_repr).unwrap(); 


        let mu_1_3_hex_repr=vec_one_1[2].to_string();
        let mu_1_3=FieldElement::from_hex(mu_1_3_hex_repr).unwrap(); 

        let mu_1_4_hex_repr=vec_one_1[3].replace(")","");
        let mu_1_4=FieldElement::from_hex(mu_1_4_hex_repr).unwrap();

        let mu_1_fromhex=(mu_1_1,mu_1_2,mu_1_3,mu_1_4);

        let vec_string: Vec<&str>=signature_string.split(r#" } })comma "#).collect();

        // let string_messO = vec_input[3].replace("comma", ",");
        let string_mess = vec_string[1].replace(") ","").replace("comma", ",");
        // println!("messss?{:?}", string_messO); 
        // println!("messss?{:?}", string_mess); 

        let messageA = messageClearing(string_mess);
        
        // let mu_1 = fixMU(signature_string.to_string());

    
        // //Hasher
        // // let hash= DefaultHasher(SipHasher13 { hasher: Hasher { k0: 0, k1: 0, length: 0, state: State { v0: 8317987319222330741, v2: 7816392313619706465, v1: 7237128888997146477, v3: 8387220255154660723 }, tail: 0, ntail: 0, _marker: PhantomData } };
        let hasher=DefaultHasher::new();


        let verified_signature_1=GVerify(group_public_key.clone(),mu_1_fromhex.clone(),hasher.clone(), messageA.clone());


        println!("verified_signature_13: {:?}", verified_signature_1);


    }
 

    // if (vec_input[0]=="GJoin"){

    //     let count_msgs = 1;
    //     let label="test".as_bytes();
    //     let (gpk, gmsk) = GSetup(count_msgs,label);

    //     //temp fix
    //     let count_msgs2 = 1;
    //     let label2="test".as_bytes();
    //     let (upk_1, usk_1)=PKIJoin(count_msgs2,label2);
    //     //arg
    //     // let user_id=1;
    //     let user_id=vec_input[1].parse::<usize>().unwrap();

    //     let (secret_register_1,gsk_1) = GJoin (user_id,gpk.clone(),gmsk.clone(), upk_1,usk_1);
        
    //     println!("Your secret: {:?} ",gsk_1);
    //     println!("Group Public: {:?} ",gpk.clone());

    // }

    if (vec_input[0]=="GJoin"){

        let count_msgs = 1;
        let label="test".as_bytes();
        let (gpk, gmsk) = GSetup(count_msgs,label);

        //temp fix
        let count_msgs2 = 1;
        let label2="test".as_bytes();
        let (upk_1, usk_1)=PKIJoin(count_msgs2,label2);
        //arg
        // let user_id=1;
        let user_id=vec_input[1].parse::<usize>().unwrap();

        let (secret_register_1,gsk_1) = GJoin2 (user_id,gpk.clone(),gmsk.clone(), upk_1,usk_1);
        
        println!("Your secret: {:?} ",gsk_1);
        println!("Group Public: {:?} ",gpk.clone());

    }

    if (vec_input[0]=="GSign"){


        // let string_ski = vec_input[1].replace("comma", ",");

        let gsk_string=vec_input[1].replace("\'","");
        let vec_one_1: Vec<&str>=gsk_string.split(r#"comma "#).collect();

        let ski_hex_repr=vec_one_1[0].replace("(","");
        let ski=FieldElement::from_hex(ski_hex_repr).unwrap(); 
        

        let g1_1_hex_repr=vec_one_1[1].replace("(","");
        let g1_1=G1::from_hex(g1_1_hex_repr).unwrap();

        let g1_2_hex_repr=vec_one_1[2].replace(")","");
        let g1_2=G1::from_hex(g1_2_hex_repr).unwrap();


        let gsk_GT_hex_repr=vec_one_1[3].replace(") ","");
        // let gsk_GT=GT::from_hex(gsk_GT_hex_repr).unwrap();

        let string_mess = vec_input[2].replace("comma", ",");

        // println!("{:?}", string_mess);

        let gsk=(ski,(g1_1,g1_2),gsk_GT_hex_repr);


        let string_gpk = vec_input[3].replace("comma", ",");
        let group_public_key=deserialize_GroupPublicKey(string_gpk);

        let (mu_1,hash_for_tuple_1, msg_1)= GSign2(group_public_key.clone(),gsk.clone(),string_mess.clone());
        println!("Signature is: {:?} \n", (mu_1,hash_for_tuple_1, msg_1));


        // // ///////////////////////////////testing verify
        // // let messageA= String::from(r#"{transaction: test, tester: sen}"#);
        // // if (messagezz.clone()==messageA){
        // //     println!("equal");
        // // }
        // // else {
        // //     println!("not equal");
        // //     println!("{:?}", messageA);
        // //     println!("{:?}", messagezz);
        // // }

        // let string_gpk = vec_input[3].replace("comma", ",");
        // let group_public_key=deserialize_GroupPublicKey(string_gpk);

        // let (mu_1,hash_for_tuple_1, msg_1)= GSign2(group_public_key.clone(),gsk.clone(),string_mess.clone());
        // // println!("Signature is: {:?} \n", (mu_1,hash_for_tuple_1, msg_1));

        // // println!("gsk_1 is: {:?} \n", gsk_1.clone());
        // // println!("Gpk is: {:?} \n", group_public_key);
        // // println!("Signature is: {:?} \n", mu_1);
        // // println!("Hasher is: {:?} \n", hash_for_tuple_1);


        // // print_type_of(&group_public_key.clone());
        // // print_type_of(&mu_1.clone());
        // // print_type_of(&hash_for_tuple_1.clone());
        // // print_type_of(&messagezz.clone());


  
        // let mu_1_1=G1::from_hex(mu_1.0).unwrap();
        // let mu_1_2=G1::from_hex(mu_1.1).unwrap();
        // let mu_1_3=FieldElement::from_hex(mu_1.2).unwrap();
        // let mu_1_4=FieldElement::from_hex(mu_1.3).unwrap();

        // let mu_1_fromhex=(mu_1_1,mu_1_2,mu_1_3,mu_1_4);



        // // let teststringer = "Gpk { g: G1 { value: ECP: [ FP: [ BIG: [ 176827B765AC12FFF95F1191D8C8AA2EC805EAF1BCE7710E2F6ED3AACAF125E1FAC1E6AC9C32B8925D77E26A5C9CAEEC ] ]comma FP: [ BIG: [ 0814B11A08E2E3014E958D84B74D0DAB030541EC7794EAC69D052CDA6926C9A2C53B63A59774553B81EA1691A46D9B22 ] ]comma FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }comma g_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 04BC772E1ED2D6B1BA5EE7CD66CCD9D16F5FA64A9876657FD9FB996EB60E096EF41C7BC96AA71E85C12478931EDE5C19 ] ]comma FP: [ BIG: [ 15D71FCB9F9BAABCCA2B622B4C2FF7658626375728546DDF32F3E4F91DDDE9DD149B40C1CA942A2AABDFC25C3443D767 ] ] ]comma FP2: [ FP: [ BIG: [ 0CC195A47A6A4F0329F4304C232C5C49121EFD8F53267B45BE721A7898EA863A1C4F2155A5C0ED4A31F33D126D15655D ] ]comma FP: [ BIG: [ 0F6BE700BDE004D83FC202E5DA8BFED0D9C04EDC76853FDEA0D1CA1B74BB11D5D69F261679ED69850BA6441D0B0F2B76 ] ] ]comma FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ]comma FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] }comma X_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 0EF0C3096DA5910C3B4671DE17A34CD9A1417A3509AED87E78BA7CCA592F71BD7B218F9B08D2BA102A79316BA0436E71 ] ]comma FP: [ BIG: [ 16852C5DAF5D9C0BDBD6CE2C8D9F23D0CF372AE59CD8AB643D260FBEFFB907DC31C52A9C43F61D9ABEF43075E7BDFA85 ] ] ]comma FP2: [ FP: [ BIG: [ 11C97BEE27F133FD2140894F726AA7B2C55D37E592B1F84E21A4097406CC084B75DB04F90620AD9A326943D913700CC8 ] ]comma FP: [ BIG: [ 0FA9357A8267E8BBC4A57C8ED0ABA6E7E7F6B5C293BE08530C6F35D453700A2221C49E3818336319776AB96DB185E3A1 ] ] ]comma FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ]comma FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] }comma Y_tilde: G2 { value: ECP2: [ FP2: [ FP: [ BIG: [ 15FB546F85D556DC43212BC33FBAD8BDB8A7FE0D996D141ED8B638FFA2273AB501F3C7B56BB5A382FAD6090C27893493 ] ]comma FP: [ BIG: [ 1289DCCA838A49C0A6CEB68F3625A0B80D5DB576C3E2947C07B6B7CA020BB06F08E1F13A57BB10CC4547339FDA4260D8 ] ] ]comma FP2: [ FP: [ BIG: [ 0EA39A3C1CC29B5AA33DC912DFC8AAE348AA68575936BB73FB4E736951BD361BD84DE6D3FF68FBE153C81E77D4CCFC15 ] ]comma FP: [ BIG: [ 0BB110EE5CFA3493893EFA58ED9694B263131D7079AAE67760A3274BC4A82941D546041D3EF22A8FB306B7C1D65E23DF ] ] ]comma FP2: [ FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ]comma FP: [ BIG: [ 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] ] ] ] } } ";
        // // let testmumu = "((G1 { value: ECP: [ FP: [ BIG: [ 08B04FBD715E3553E7DCF2D79979EC0F944CF26E58B71ED9A050B5FB11620F821B8CEFE43C97D04CA1AA4D6A1A88A3A4 ] ], FP: [ BIG: [ 19EB8E3680C9804FA9DE93F2F9405359A799C604A68DE6AFFAE548CC3D76355AE2F01A0AABB900F2130EDC2EC62DF11F ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }, G1 { value: ECP: [ FP: [ BIG: [ 145E6AB325A5A1A7BC645E2CE6966451C9D4B3231D02C39E92A250657FEB7B7F57BF36761710D0BB9766E72FB992CAC7 ] ], FP: [ BIG: [ 021FF7CB8F6BC6ADA04FCF32150972803694A9778ECD7A8254DDDD9B1B592B1351586A411016DCEB02520411FDB718A7 ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }, FieldElement { value: BIG: [ 000000000000000000000000000000001A9F40C9741E50D5F9FC67E5751E6F53174F249F7BBD9F842BE88307233B5233 ] }, FieldElement { value: BIG: [ 00000000000000000000000000000000251B78E84AD0E95F6738E5544315729B9AE0CDD9E54CFF4B0562A1FE1ABD4561 ] }) ";
        // // // let testmumu2 = "((G1 { value: ECP: [ FP: [ BIG: [ 08B04FBD715E3553E7DCF2D79979EC0F944CF26E58B71ED9A050B5FB11620F821B8CEFE43C97D04CA1AA4D6A1A88A3A4 ] ], FP: [ BIG: [ 19EB8E3680C9804FA9DE93F2F9405359A799C604A68DE6AFFAE548CC3D76355AE2F01A0AABB900F2130EDC2EC62DF11F ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }, G1 { value: ECP: [ FP: [ BIG: [ 145E6AB325A5A1A7BC645E2CE6966451C9D4B3231D02C39E92A250657FEB7B7F57BF36761710D0BB9766E72FB992CAC7 ] ], FP: [ BIG: [ 021FF7CB8F6BC6ADA04FCF32150972803694A9778ECD7A8254DDDD9B1B592B1351586A411016DCEB02520411FDB718A7 ] ], FP: [ BIG: [ 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84 ] ] ] }, FieldElement { value: BIG: [ 000000000000000000000000000000001A9F40C9741E50D5F9FC67E5751E6F53174F249F7BBD9F842BE88307233B5233 ] }, FieldElement { value: BIG: [ 00000000000000000000000000000000251B78E84AD0E95F6738E5544315729B9AE0CDD9E54CFF4B0562A1FE1ABD4561 ] }) ";

        // let testmeage = "{transaction: testcomma tester: sen}";

        // // let string_gpk = teststringer.replace("comma", ",");
        // // let mu_12 = fixMU(testmumu.to_string());
        // // // let mu_13 = fixMU(testmumu2.to_string());


        // let s = FieldElement::random(); 
        // // let s = FieldElement::one();

        // let hex_repr = mu_1_fromhex.0.to_hex();
        // println!("hex_repr:{:?}\n", hex_repr);
        // let hex_string_mu1="1 1864000FFA211971EFC7B03097662A9CFE2AC9EC54A7925BB2DD81C1A26812E4C28C71CD0942C2E0205A1A6E6889282F 1 1153700E2CD7A5E558AB383657F278B08269BFA7B69FC0984CD05A3E6AD799745F5323260BB844430B444766CB5168F7 2 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84";


        // println!("G1_mu_1 scalar mult test: {:?} \n", mu_1_fromhex.0.scalar_mul_variable_time(&s));

        
        // let mu_12 = G1::from_hex(hex_string_mu1.to_string()).unwrap(); 
        // println!("G1_mu_2 scalar mult test: {:?} \n", mu_12.scalar_mul_variable_time(&s));

        // let mu_3 = G1::from_hex(hex_repr).unwrap(); 
        // println!("G1_mu_3 scalar mult test: {:?} \n", mu_3.scalar_mul_variable_time(&s));


        // // let mu_4 = G1::from_hex("1 08B04FBD715E3553E7DCF2D79979EC0F944CF26E58B71ED9A050B5FB11620F821B8CEFE43C97D04CA1AA4D6A1A88A3A4 1 19EB8E3680C9804FA9DE93F2F9405359A799C604A68DE6AFFAE548CC3D76355AE2F01A0AABB900F2130EDC2EC62DF11F 2 13317C30F3A0D636D56A23C34FDD80B891ECBDE7C2B7D6E16B0F4B0B7E6D26CB6147ACDE629C4A23C57400D203A9FB84".to_string()).unwrap(); 
        // // println!("G1_mu_4 scalar mult test: {:?} \n", mu_4.scalar_mul_variable_time(&s));

        // // // g.scalar_mul_variable_time(&a);

        // // if (mu_1==mu_12){
        // //     println!("equal!");
        // // }
  

        // // let group_public_key2=deserialize_GroupPublicKey(string_gpk);
        // let messagezz12 = messageClearing(testmeage.replace("comma", ",").to_string());
        // let verified_signature_1=GVerify2(group_public_key.clone(),mu_1_fromhex.clone(),hash_for_tuple_1.clone(), messagezz12.clone());
        // println!("verified_signature_1 is: {:?} \n", verified_signature_1);


        // // let verified_signature_2=GVerify2(group_public_key2.clone(),mu_12.clone(),hash_for_tuple_1.clone(), messagezz12.clone());
        // // println!("verified_signature_2 is: {:?} \n", verified_signature_2);



    }


    if (vec_input[0]=="PKIJoin"){
        //temp fix
        let count_msgs = 1;
        let label="test".as_bytes();

        let (upk_1, usk_1)=PKIJoin(count_msgs,label);
        //arg
        // let user_id=1;
        let label=vec_input[1].as_bytes();
        let (upk_1, usk_1)=PKIJoin(count_msgs,label);
        println!("Your secret: {:?} ",usk_1);
    }



}
