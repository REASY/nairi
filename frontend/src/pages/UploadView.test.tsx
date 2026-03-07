import {render, screen, fireEvent, waitFor} from '@testing-library/react';
import {describe, it, expect} from 'vitest';
import UploadView from './UploadView';

describe('UploadView', () => {
    it('renders initial state with analyse button', () => {
        render(<UploadView/>);
        expect(screen.getByText('Upload APK for Analysis')).toBeInTheDocument();
        expect(screen.getByRole('button', {name: /Analyse/i})).toBeInTheDocument();
    });

    it('clicking analyse transitions to active run status', async () => {
        render(<UploadView/>);
        const button = screen.getByRole('button', {name: /Analyse/i});

        // Initial state
        expect(screen.queryByText('Active Run Status')).not.toBeInTheDocument();

        // Click action
        fireEvent.click(button);

        // Should show active run status
        expect(screen.getByText('Active Run Status')).toBeInTheDocument();
        expect(screen.getByText('Initializing...')).toBeInTheDocument();
    });
});
