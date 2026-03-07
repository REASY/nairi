import {http, HttpResponse, delay} from 'msw'

export const handlers = [
    // Mock config endpoint
    http.get('/api/config', () => {
        return HttpResponse.json({
            staticImage: 'nairi-static:latest',
            adbTarget: '127.0.0.1:5555'
        })
    }),

    // Mock analysis start endpoint
    http.post('/api/analyze', async () => {
        await delay(1000)
        return HttpResponse.json({
            runId: 'run-' + Math.random().toString(36).substr(2, 9),
            status: 'queued'
        }, {status: 201})
    }),

    // Mock analysis status endpoint
    http.get('/api/runs/:runId', ({params}) => {
        // Return a mocked progress timeline
        return HttpResponse.json({
            runId: params.runId,
            status: 'running',
            stages: [
                {name: 'Static Analysis', status: 'completed', details: 'apktool & ghidra finished'},
                {name: 'Runtime Analysis', status: 'running', details: 'redroid sandbox active (65%)'},
                {name: 'Network MITM', status: 'pending', details: 'Waiting for runtime'}
            ]
        })
    })
]
