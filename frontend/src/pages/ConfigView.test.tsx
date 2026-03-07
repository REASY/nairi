import {render, screen, waitFor} from '@testing-library/react';
import {describe, it, expect} from 'vitest';
import ConfigView from './ConfigView';

describe('ConfigView', () => {
    it('renders loading state initially', () => {
        render(<ConfigView/>);
        expect(screen.getByText(/loading configuration/i)).toBeInTheDocument();
    });

    it('renders config form once loaded', async () => {
        render(<ConfigView/>);

        // MSW will return our mocked config
        await waitFor(() => {
            expect(screen.getByText('System Configuration')).toBeInTheDocument();
        });

        // We expect the config inputs to be populated
        const inputs = screen.getAllByRole('textbox');
        expect(inputs).toHaveLength(2);
    });
});
