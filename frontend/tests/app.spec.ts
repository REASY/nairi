import {test, expect} from '@playwright/test';

test('has title', async ({page}) => {
    await page.goto('http://localhost:5173/');

    // Expect a title "to contain" a substring.
    await expect(page).toHaveTitle(/NAIRI/);
});

test('can navigate to analysis and upload', async ({page}) => {
    await page.goto('http://localhost:5173/');

    // Click the get started link.
    await page.getByRole('button', {name: 'Analyse'}).click();

    // Expects page to have a heading with the name of Installation.
    await expect(page.getByText('Active Run Status')).toBeVisible();
});

test('can navigate to configuration', async ({page}) => {
    await page.goto('http://localhost:5173/');

    // Click config link
    await page.getByRole('link', {name: 'Configuration'}).click();

    // Verify headers
    await expect(page.getByRole('heading', {name: 'System Configuration'})).toBeVisible();

    // Test the input fields populate
    // Wait for network mock to resolve by waiting for input value
    const input = page.locator('input').first();
    await expect(input).toHaveValue(/nairi-static:latest|127.0.0.1:5555/);
});
