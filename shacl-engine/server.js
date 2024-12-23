import express from 'express';
import { Validator } from 'shacl-engine'

const PORT = 3000;

const app = express();
app.use(express.json());

const validateRequest = (req, res, next) => {
    if (!req.body.data) {
        return res.status(400).json({
            message: 'data field required!',
        });
    }

    next();
};

app.post('/validate', validateRequest, (req, res) => {
    const { data, shape } = req.body;
    console.log('Data:', data);
    console.log('Shape:', shape);

    res.status(200).json({
        message: 'Validation successful!',
        data,
        shape,
    });
});

app.all('*', (req, res) => {
    res.status(404).send({
        message: 'This is the catch-all route!',
        method: req.method,
        path: req.originalUrl,
    });
});

app.listen(PORT, () => {
    console.log(`Server running at http://localhost:${PORT}`);
});
