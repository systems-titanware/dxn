// Script Function Example: Wallet Transform
// TypeScript script for quick transformations with type safety

export function transformAddress(address: string): string {
    // Convert to lowercase and add prefix if missing
    let transformed = address.toLowerCase();
    if (!transformed.startsWith("0x")) {
        transformed = "0x" + transformed;
    }
    return transformed;
}

export function formatBalance(balance: number): string {
    // Format balance with unit
    return `${balance} wei`;
}

export function validateAndFormat(address: string, balance: number): { address: string; balance: string; valid: boolean } {
    // Combined validation and formatting
    const valid = address.length > 10;
    return {
        address: transformAddress(address),
        balance: formatBalance(balance),
        valid: valid
    };
}

