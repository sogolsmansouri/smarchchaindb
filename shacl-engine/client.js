import axios from 'axios';

const postData = {
    '@context': {
        'ex': 'http://example.org/', 
        'schema': 'http://schema.org/', 
        'transaction_id': 'ex:transaction_id',
        'operation': 'ex:operation',
        'status': 'ex:status',
        'ref': {
            '@id': 'ex:ref',
            '@type': '@id'
        }
    },
    '@id': 'http://example.org/txn/eacec928903a0996d204747a33b93ea0c4f897716882882278ea184b1dacc9e7',
    '@type': 'ex:ADV',
    'ref': 'http://example.org/txn/274ed4418e0a19b80614b13910025a3b5c7b0075cfb695e72659c7a66f9c6025',
    'status': 'Open'
}

axios.post('http://localhost:3000/validate', postData)
    .then((response) => {
        console.log('Response:', response.data);
    })
    .catch((error) => {
        console.error('Error:\n', error.response.data);
    });
