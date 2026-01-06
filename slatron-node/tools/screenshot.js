const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

if (process.argv.length < 4) {
    console.error('Usage: node screenshot.js <url> <output_path>');
    process.exit(1);
}

const url = process.argv[2];
const outputPath = process.argv[3];

(async () => {
    try {
        const browser = await puppeteer.launch({
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
        const page = await browser.newPage();

        await page.setViewport({ width: 1920, height: 1080 });

        console.log(`Navigating to ${url}...`);
        await page.goto(url, { waitUntil: 'networkidle2', timeout: 30000 });

        console.log(`Taking screenshot to ${outputPath}...`);

        // Ensure directory exists
        const dir = path.dirname(outputPath);
        if (!fs.existsSync(dir)){
            fs.mkdirSync(dir, { recursive: true });
        }

        await page.screenshot({ path: outputPath, fullPage: false });

        await browser.close();
        console.log('Screenshot taken successfully.');
    } catch (error) {
        console.error('Error taking screenshot:', error);
        process.exit(1);
    }
})();
