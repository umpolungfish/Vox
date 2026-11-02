"""Resume the whole-chain root readout from an exact shared-port compilation."""
import argparse
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import subprocess


def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda:f.read(8*1024*1024),b''):h.update(block)
    return h.hexdigest()


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('prepared',type=Path)
    parser.add_argument('contraction',type=Path)
    parser.add_argument('output',type=Path)
    parser.add_argument('--witness',type=Path,
        help='reuse a source-bound stationary witness and resume at interval certification')
    args=parser.parse_args()
    prepared=json.loads(args.prepared.read_text())
    compiled=json.loads(args.contraction.read_text())
    if compiled.get('schema')!='whole-native-algebraic-fold/v1' or compiled.get('status')!='compiled':
        raise ValueError('resume input is not a completed shared-port compilation')
    if digest(args.prepared)!=compiled.get('source_sha256'):
        raise ValueError('shared-port compilation belongs to a different prepared source')
    construction=compiled['construction'];event=construction['shared_source_event_tensor']
    contraction=construction['shared_residue_port_contraction']
    if (event['event_count']!=contraction['source_events']
        or event['pair_tensor_uses']!=contraction['pair_tensor_occurrence_uses']
        or event['factorized_event_tensor_sha256']!=contraction['factorized_event_tensor_sha256']):
        raise ValueError('contraction metadata lost its shared event ancestry')
    responses=[]
    for left_residue,left_form,right_residue,right_form,coefficients in contraction['response_coefficients']:
        responses.append((left_residue,left_form,right_residue,right_form,
            *(Fraction(int(numerator),int(denominator)) for numerator,denominator in coefficients)))
    source={
        'source_paths':prepared['source_paths'],
        'pair_contraction_windings':prepared['one_shot_lattice_contraction']['residue_character_windings'],
        'shared_source_event_readout':event['events'],
        'factorized_shared_event_tensor':event['factorized_event_tensors'],
        'source_four_coordinates':event['source_four_coordinates'],
        'contracted_source_four_coordinates':event['contracted_source_four_coordinates'],
        'shared_port_responses':responses,
        'shared_port_contraction':contraction,
        'source_event_occurrence_uses':contraction['pair_tensor_occurrence_uses'],
        'all_source_occurrences_rejoined':event['all_occurrences_rejoined'],
        'source_sha256':compiled['source_sha256'],
        'pairs':[{'residues':pair['residues'],'spatial_channels':pair['spatial_channels']}
            for pair in prepared['pairs']],
    }
    from whole_algebraic_readout import solve
    from whole_algebraic_readout import source_bank_records
    result=compiled.copy()
    bank_rows=source_bank_records(source['pairs'])
    result['construction']['carried_bank_tensor']={
        'channels':len(bank_rows),
        'records_sha256':hashlib.sha256(json.dumps(bank_rows,separators=(',',':')).encode()).hexdigest(),
        'role':'source-addressed preference/release readout carried through SIC; excluded from independent pair force terms'}
    result.update(solve(source,resume_witness=args.witness,
        witness_path=args.output.with_suffix('.witness.json')))
    result['construction']['shared_residue_port_contraction']=contraction
    result['construction']['shared_source_event_tensor']['contracted_source_four_coordinates']=event['contracted_source_four_coordinates']
    args.output.write_text(json.dumps(result,indent=2)+'\n')
    if result.get('closure_verified') is True:
        binary=Path(__file__).resolve().parent/'target/x86_64-unknown-linux-musl/release/vox'
        subprocess.run([str(binary),'compile','--whole-algebraic-return',str(args.output)],check=True)
    else:
        print('Whole algebraic family remains held; source-bound solver diagnostics written',flush=True)


if __name__=='__main__':main()
